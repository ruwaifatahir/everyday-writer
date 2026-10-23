//! The rewrite session: the one controller that owns a rewrite from the
//! shortcut to the popup closing.
//!
//! It is the only module that knows the order of the steps. Everything it
//! touches goes through four ports, so tests drive it with fakes and no
//! Windows, network or key:
//!
//! - [`TextBridge`]: captures the selection and copies text;
//! - [`Rewriter`]: rewrites the selection;
//! - [`SettingsView`]: the active instruction set;
//! - [`PopupView`]: shows each [`PopupState`].
//!
//! Inputs are [`Event`]s, fed to [`RewriteSession::handle`].
//!
//! Privacy: the selection and the result are user data. They are never
//! logged, and [`PopupState`]'s `Debug` output hides them.

use std::future::{Future, poll_fn};
use std::pin::pin;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::task::Poll;
use std::time::Duration;

use serde::Serialize;
use tokio::sync::oneshot;

use crate::llm::{RewriteError, Rewriter};
use crate::settings::SettingsView;

/// Something the user did that the session reacts to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// The rewrite shortcut was pressed.
    ShortcutPressed,
    /// Replace was chosen (the button or Enter).
    Replace,
    /// Copy was chosen.
    Copy,
    /// The popup was closed: ✕, Esc or a click outside it.
    Close,
    /// The message's Open Settings action was chosen.
    OpenSettings,
}

/// What the popup shows. Serialized for the popup window as
/// `{ "kind": "working" }`, `{ "kind": "result", "text": ... }` and so on.
#[derive(Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PopupState {
    /// The popup is hidden.
    Hidden,
    /// A rewrite is in progress.
    Working,
    /// The result, exactly as the rewrite returned it.
    Result {
        /// The result. User data: never log it.
        text: String,
    },
    /// A short message instead of a result.
    Message {
        /// The message. Never holds the selection or the result.
        text: String,
        /// A button offered with the message, if any.
        action: Option<MessageAction>,
    },
}

/// Hides the result, which is user data.
impl std::fmt::Debug for PopupState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hidden => f.write_str("Hidden"),
            Self::Working => f.write_str("Working"),
            Self::Result { .. } => f.write_str("Result { .. }"),
            Self::Message { text, action } => f
                .debug_struct("Message")
                .field("text", text)
                .field("action", action)
                .finish(),
        }
    }
}

/// A button a message can offer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MessageAction {
    /// Opens the settings window.
    OpenSettings,
}

/// What a capture found: the selection and the window it came from.
pub struct Captured<W> {
    /// The selected text. User data: never log it.
    pub selection: String,
    /// The window the selection came from, where Replace goes back to.
    pub window: W,
}

/// The text bridge failed. Holds no clipboard content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("the clipboard could not be used")]
pub struct BridgeError;

/// Reads the selection from, and puts text into, other apps.
pub trait TextBridge: Send + Sync {
    /// Identifies the window a selection came from.
    type Window: Send + Sync;

    /// Captures the selection in the foreground app, leaving the user's
    /// clipboard as it was. `None` means nothing was selected (or capture
    /// failed, which looks the same to the user).
    fn capture(&self) -> impl Future<Output = Option<Captured<Self::Window>>> + Send;

    /// Puts `text` on the clipboard.
    ///
    /// # Errors
    ///
    /// Returns [`BridgeError`] if the clipboard can't be written.
    fn copy(&self, text: &str) -> impl Future<Output = Result<(), BridgeError>> + Send;
}

/// Shows the popup states the session decides on.
pub trait PopupView: Send + Sync {
    /// Shows `state`, or hides the popup for [`PopupState::Hidden`]. Called
    /// only when the state changes.
    fn render(&self, state: &PopupState);

    /// Opens the settings window.
    fn open_settings(&self);
}

impl<T: PopupView + ?Sized> PopupView for Arc<T> {
    fn render(&self, state: &PopupState) {
        (**self).render(state);
    }

    fn open_settings(&self) {
        (**self).open_settings();
    }
}

/// The longest selection sent for a rewrite, in characters. A constant, not
/// a setting.
pub const MAX_SELECTION_CHARS: usize = 5_000;

/// How long a rewrite may take before it is cancelled. A constant, not a
/// setting.
pub const REWRITE_TIMEOUT: Duration = Duration::from_secs(30);

// The popup's messages. None carries request or response content.
const TOO_LONG: &str = "This text is too long. Select up to 5,000 characters.";
const NO_KEY: &str = "Add your API key to get started";
const TIMED_OUT: &str = "The AI took too long. Try again.";
const NO_INTERNET: &str = "Can't reach the AI. Check your internet connection.";
const KEY_REJECTED: &str = "Your API key was rejected. Check it in Settings.";
const KEY_UNREADABLE: &str = "Couldn't read your saved API key. Check it in Settings.";
const BAD_SETTINGS: &str = "Your AI settings need a look.";
const RATE_LIMITED: &str = "Too many requests right now. Wait a moment and try again.";
const PROVIDER_FAILED: &str = "The AI service had a problem. Try again.";
const EMPTY_RESPONSE: &str = "The AI sent back nothing. Try again.";
const REWRITE_FAILED: &str = "Couldn't rewrite your text. Try again.";

/// The session's own state, richer than the popup state it shows.
enum Phase<W> {
    /// The popup is hidden and nothing is going on.
    Hidden,
    /// A capture is in progress; the popup is still hidden.
    Capturing,
    /// The popup is open for `captured`.
    Open { captured: Captured<W>, view: View },
}

/// What an open popup shows.
enum View {
    /// Rewrite number `run` is in progress. Dropping `cancel` cancels it.
    Working {
        run: u64,
        #[expect(dead_code, reason = "only dropped, which cancels the rewrite")]
        cancel: oneshot::Sender<()>,
    },
    /// The rewrite returned this result.
    Result(String),
    /// A message is shown: always a [`PopupState::Message`].
    Message(PopupState),
}

impl View {
    /// The popup state this view shows.
    fn shown(&self) -> PopupState {
        match self {
            Self::Working { .. } => PopupState::Working,
            Self::Result(text) => PopupState::Result { text: text.clone() },
            Self::Message(message) => message.clone(),
        }
    }
}

/// A rewrite that is ready to send.
struct Working {
    run: u64,
    instructions: String,
    selection: String,
    cancelled: oneshot::Receiver<()>,
}

struct State<W> {
    phase: Phase<W>,
    /// Numbers each rewrite, so a late answer can't update a newer popup.
    next_run: u64,
}

/// Runs rewrites: capture, rewrite, show, then Copy, Replace or close.
pub struct RewriteSession<B: TextBridge, R, S, P> {
    bridge: B,
    rewriter: R,
    settings: S,
    popup: P,
    state: Mutex<State<B::Window>>,
}

impl<B, R, S, P> RewriteSession<B, R, S, P>
where
    B: TextBridge,
    R: Rewriter,
    S: SettingsView,
    P: PopupView,
{
    /// A session with a hidden popup.
    pub fn new(bridge: B, rewriter: R, settings: S, popup: P) -> Self {
        Self {
            bridge,
            rewriter,
            settings,
            popup,
            state: Mutex::new(State {
                phase: Phase::Hidden,
                next_run: 0,
            }),
        }
    }

    /// Reacts to `event`. A shortcut press returns once its rewrite has
    /// finished or was cancelled; meanwhile other events are handled
    /// concurrently.
    pub async fn handle(&self, event: Event) {
        match event {
            Event::ShortcutPressed => self.shortcut_pressed().await,
            Event::Copy => self.copy().await,
            Event::Close => self.close(),
            Event::OpenSettings => {
                self.close();
                self.popup.open_settings();
            }
            // Replace arrives in ticket 04.
            Event::Replace => {}
        }
    }

    fn lock(&self) -> MutexGuard<'_, State<B::Window>> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Captures and rewrites when the popup is hidden. When it is open,
    /// cancels any rewrite in progress and rewrites the same selection
    /// again, without capturing.
    async fn shortcut_pressed(&self) {
        let rerun = {
            let mut state = self.lock();
            match std::mem::replace(&mut state.phase, Phase::Capturing) {
                Phase::Hidden => None,
                // Already capturing: this press adds nothing.
                Phase::Capturing => return,
                // Dropping the old view cancels its rewrite.
                Phase::Open { captured, view } => {
                    Some(self.start(&mut state, captured, &view.shown()))
                }
            }
        };
        let working = if let Some(working) = rerun {
            working
        } else {
            let captured = self.bridge.capture().await;
            self.captured(captured)
        };
        if let Some(working) = working {
            self.rewrite(working).await;
        }
    }

    /// Acts on a finished capture: hides the popup again when nothing was
    /// selected, or opens it for the selection.
    fn captured(&self, captured: Option<Captured<B::Window>>) -> Option<Working> {
        let mut state = self.lock();
        if !matches!(state.phase, Phase::Capturing) {
            // Closed while capturing.
            return None;
        }
        let Some(captured) = captured.filter(|captured| !captured.selection.trim().is_empty())
        else {
            state.phase = Phase::Hidden;
            return None;
        };
        self.start(&mut state, captured, &PopupState::Hidden)
    }

    /// Opens the popup for `captured`, which showed `before`. Shows a
    /// message and returns `None` if the selection is too long or the
    /// provider isn't ready; otherwise shows working and returns the
    /// rewrite to send, with the current active instructions.
    fn start(
        &self,
        state: &mut State<B::Window>,
        captured: Captured<B::Window>,
        before: &PopupState,
    ) -> Option<Working> {
        let refused = if captured.selection.chars().count() > MAX_SELECTION_CHARS {
            Some(message(TOO_LONG, None))
        } else if self.settings.is_provider_ready() {
            None
        } else {
            Some(message(NO_KEY, Some(MessageAction::OpenSettings)))
        };
        let (view, working) = if let Some(shown) = refused {
            (View::Message(shown), None)
        } else {
            let run = state.next_run;
            state.next_run += 1;
            let (cancel, cancelled) = oneshot::channel();
            let working = Working {
                run,
                instructions: self.settings.active_instructions(),
                selection: captured.selection.clone(),
                cancelled,
            };
            (View::Working { run, cancel }, Some(working))
        };
        let shown = view.shown();
        state.phase = Phase::Open { captured, view };
        if shown != *before {
            self.popup.render(&shown);
        }
        working
    }

    /// Sends `working` and shows its outcome, unless it is cancelled first.
    /// Gives up with a message after [`REWRITE_TIMEOUT`].
    async fn rewrite(&self, working: Working) {
        let Working {
            run,
            instructions,
            selection,
            cancelled,
        } = working;
        let rewrite = tokio::time::timeout(
            REWRITE_TIMEOUT,
            self.rewriter.rewrite(&instructions, &selection),
        );
        let Some(outcome) = until_cancelled(rewrite, cancelled).await else {
            return;
        };
        let view = match outcome {
            Ok(Ok(text)) => View::Result(text),
            Ok(Err(error)) => View::Message(message_for(&error)),
            Err(_elapsed) => View::Message(message(TIMED_OUT, None)),
        };
        self.finish(run, view);
    }

    /// Shows `outcome` for rewrite `run`, unless the popup has moved on.
    fn finish(&self, run: u64, outcome: View) {
        let mut state = self.lock();
        let Phase::Open { view, .. } = &mut state.phase else {
            return;
        };
        if !matches!(view, View::Working { run: current, .. } if *current == run) {
            return;
        }
        *view = outcome;
        self.popup.render(&view.shown());
    }

    async fn copy(&self) {
        let result = {
            let state = self.lock();
            let Phase::Open {
                view: View::Result(result),
                ..
            } = &state.phase
            else {
                return;
            };
            result.clone()
        };
        if self.bridge.copy(&result).await.is_err() {
            // The result stays shown, so Copy can be tried again.
            tracing::warn!("the result couldn't be copied");
            return;
        }
        self.close();
    }

    /// Hides the popup. Dropping the phase cancels a rewrite in progress.
    fn close(&self) {
        let mut state = self.lock();
        let was_open = matches!(state.phase, Phase::Open { .. });
        state.phase = Phase::Hidden;
        if was_open {
            self.popup.render(&PopupState::Hidden);
        }
    }
}

/// The message shown for a failed rewrite. Never includes request or
/// response content.
fn message_for(error: &RewriteError) -> PopupState {
    let open_settings = Some(MessageAction::OpenSettings);
    match error {
        RewriteError::MissingApiKey => message(NO_KEY, open_settings),
        RewriteError::KeyStore => message(KEY_UNREADABLE, open_settings),
        RewriteError::InvalidSettings(_) => message(BAD_SETTINGS, open_settings),
        RewriteError::Unauthorized => message(KEY_REJECTED, open_settings),
        RewriteError::RateLimited => message(RATE_LIMITED, None),
        RewriteError::Network => message(NO_INTERNET, None),
        RewriteError::Provider { .. } => message(PROVIDER_FAILED, None),
        RewriteError::EmptyResponse => message(EMPTY_RESPONSE, None),
        RewriteError::Other => message(REWRITE_FAILED, None),
    }
}

/// A message state showing `text`, with an optional button.
fn message(text: &str, action: Option<MessageAction>) -> PopupState {
    PopupState::Message {
        text: text.to_owned(),
        action,
    }
}

/// Runs `future` until it finishes, or returns `None` as soon as
/// `cancelled` fires or its sender is dropped. The future is dropped then,
/// which aborts the request.
async fn until_cancelled<F: Future>(
    future: F,
    mut cancelled: oneshot::Receiver<()>,
) -> Option<F::Output> {
    let mut future = pin!(future);
    poll_fn(|cx| {
        // Cancel first: an answer that is ready by now still came too late.
        if std::pin::Pin::new(&mut cancelled).poll(cx).is_ready() {
            return Poll::Ready(None);
        }
        future.as_mut().poll(cx).map(Some)
    })
    .await
}

#[cfg(test)]
mod tests;
