//! Rewrite session tests: fake ports, no Windows, network or key.

use std::future::{Future, ready};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use tokio::sync::{Notify, oneshot};

use super::{BridgeError, Captured, Event, PopupState, PopupView, RewriteSession, TextBridge};
use crate::llm::{RewriteError, Rewriter};
use crate::settings::{ModelSettings, Provider, SettingsView};

/// A bridge whose capture finds `selection`, recording what it is asked
/// to do.
#[derive(Default)]
struct FakeBridge {
    selection: Option<String>,
    copy_fails: bool,
    captures: AtomicUsize,
    copied: Mutex<Vec<String>>,
}

impl FakeBridge {
    fn selecting(selection: &str) -> Self {
        Self {
            selection: Some(selection.to_owned()),
            ..Self::default()
        }
    }

    fn copied(&self) -> Vec<String> {
        self.copied.lock().unwrap().clone()
    }
}

impl TextBridge for FakeBridge {
    type Window = u32;

    fn capture(&self) -> impl Future<Output = Option<Captured<u32>>> {
        self.captures.fetch_add(1, Ordering::Relaxed);
        ready(self.selection.clone().map(|selection| Captured {
            selection,
            window: 7,
        }))
    }

    fn copy(&self, text: &str) -> impl Future<Output = Result<(), BridgeError>> {
        if self.copy_fails {
            return ready(Err(BridgeError));
        }
        self.copied.lock().unwrap().push(text.to_owned());
        ready(Ok(()))
    }
}

/// A rewriter that answers with `answer`, or waits for the test to answer
/// through the sender from [`TestRewriter::gated`].
struct TestRewriter {
    answer: Option<Result<String, RewriteError>>,
    gate: Mutex<Option<oneshot::Receiver<Result<String, RewriteError>>>>,
    calls: Mutex<Vec<(String, String)>>,
    called: Notify,
    cancelled: AtomicBool,
}

impl TestRewriter {
    fn answering(answer: Result<String, RewriteError>) -> Self {
        Self {
            answer: Some(answer),
            gate: Mutex::new(None),
            calls: Mutex::default(),
            called: Notify::new(),
            cancelled: AtomicBool::new(false),
        }
    }

    fn returning(result: &str) -> Self {
        Self::answering(Ok(result.to_owned()))
    }

    /// A rewriter that answers only when the returned sender is used.
    fn gated() -> (Self, oneshot::Sender<Result<String, RewriteError>>) {
        let (answer, gate) = oneshot::channel();
        let rewriter = Self {
            answer: None,
            gate: Mutex::new(Some(gate)),
            ..Self::returning("")
        };
        (rewriter, answer)
    }

    fn calls(&self) -> Vec<(String, String)> {
        self.calls.lock().unwrap().clone()
    }
}

/// Marks the rewrite as cancelled if it is dropped before it answers.
struct CancelGuard<'a>(&'a AtomicBool, bool);

impl Drop for CancelGuard<'_> {
    fn drop(&mut self) {
        if !self.1 {
            self.0.store(true, Ordering::Relaxed);
        }
    }
}

impl Rewriter for TestRewriter {
    async fn rewrite(&self, instructions: &str, text: &str) -> Result<String, RewriteError> {
        self.calls
            .lock()
            .unwrap()
            .push((instructions.to_owned(), text.to_owned()));
        let mut guard = CancelGuard(&self.cancelled, false);
        self.called.notify_one();
        let gate = self.gate.lock().unwrap().take();
        let answer = match gate {
            Some(gate) => gate.await.unwrap_or(Err(RewriteError::Other)),
            None => self.answer.clone().unwrap_or(Err(RewriteError::Other)),
        };
        guard.1 = true;
        answer
    }
}

const INSTRUCTIONS: &str = "Fix mistakes only.";

struct FakeSettings;

impl SettingsView for FakeSettings {
    fn model_settings(&self) -> ModelSettings {
        ModelSettings {
            provider: Provider::OpenAi,
            model: "gpt-test".to_owned(),
        }
    }

    fn is_provider_ready(&self) -> bool {
        true
    }

    fn active_instructions(&self) -> String {
        INSTRUCTIONS.to_owned()
    }
}

/// Records every popup state it is asked to show.
#[derive(Default)]
struct RecordingPopup {
    states: Mutex<Vec<PopupState>>,
    settings_opened: AtomicUsize,
}

impl PopupView for RecordingPopup {
    fn render(&self, state: &PopupState) {
        self.states.lock().unwrap().push(state.clone());
    }

    fn open_settings(&self) {
        self.settings_opened.fetch_add(1, Ordering::Relaxed);
    }
}

type TestSession = RewriteSession<FakeBridge, TestRewriter, FakeSettings, RecordingPopup>;

fn session(bridge: FakeBridge, rewriter: TestRewriter) -> TestSession {
    RewriteSession::new(bridge, rewriter, FakeSettings, RecordingPopup::default())
}

fn states(session: &TestSession) -> Vec<PopupState> {
    session.popup.states.lock().unwrap().clone()
}

fn result(text: &str) -> PopupState {
    PopupState::Result {
        text: text.to_owned(),
    }
}

/// Presses the shortcut on a session whose rewrite returns at once.
async fn rewritten(selection: &str, result: &str) -> TestSession {
    let session = session(
        FakeBridge::selecting(selection),
        TestRewriter::returning(result),
    );
    session.handle(Event::ShortcutPressed).await;
    session
}

#[tokio::test]
async fn a_rewrite_shows_working_then_the_result() {
    let session = rewritten("helo wrld", "Hello, world.").await;

    assert_eq!(
        states(&session),
        [PopupState::Working, result("Hello, world.")]
    );
}

#[tokio::test]
async fn the_rewriter_gets_the_selection_and_the_active_instructions() {
    let session = rewritten("helo wrld", "Hello, world.").await;

    assert_eq!(
        session.rewriter.calls(),
        [(INSTRUCTIONS.to_owned(), "helo wrld".to_owned())]
    );
}

#[tokio::test]
async fn nothing_selected_leaves_the_popup_hidden() {
    let session = session(FakeBridge::default(), TestRewriter::returning("unused"));

    session.handle(Event::ShortcutPressed).await;

    assert_eq!(states(&session), []);
    assert_eq!(session.rewriter.calls(), []);
}

#[tokio::test]
async fn a_blank_selection_counts_as_nothing_selected() {
    let session = session(
        FakeBridge::selecting(" \r\n\t"),
        TestRewriter::returning("unused"),
    );

    session.handle(Event::ShortcutPressed).await;

    assert_eq!(states(&session), []);
    assert_eq!(session.rewriter.calls(), []);
}

#[tokio::test]
async fn an_unchanged_result_is_still_shown_as_a_result() {
    let session = rewritten("Hello, world.", "Hello, world.").await;

    assert_eq!(
        states(&session),
        [PopupState::Working, result("Hello, world.")]
    );
}

#[tokio::test]
async fn copy_puts_the_result_on_the_clipboard_and_hides_the_popup() {
    let session = rewritten("helo wrld", "Hello, world.").await;

    session.handle(Event::Copy).await;

    assert_eq!(session.bridge.copied(), ["Hello, world."]);
    assert_eq!(states(&session).last(), Some(&PopupState::Hidden));
}

#[tokio::test]
async fn a_failed_copy_keeps_the_result_shown() {
    let session = session(
        FakeBridge {
            copy_fails: true,
            ..FakeBridge::selecting("helo wrld")
        },
        TestRewriter::returning("Hello, world."),
    );
    session.handle(Event::ShortcutPressed).await;

    session.handle(Event::Copy).await;

    assert_eq!(
        states(&session),
        [PopupState::Working, result("Hello, world.")]
    );
}

#[tokio::test]
async fn copy_does_nothing_while_working() {
    let (rewriter, _answer) = TestRewriter::gated();
    let session = session(FakeBridge::selecting("helo wrld"), rewriter);

    tokio::join!(session.handle(Event::ShortcutPressed), async {
        session.rewriter.called.notified().await;
        session.handle(Event::Copy).await;
        session.handle(Event::Close).await;
    });

    assert_eq!(session.bridge.copied(), Vec::<String>::new());
}

#[tokio::test]
async fn close_hides_the_result() {
    let session = rewritten("helo wrld", "Hello, world.").await;

    session.handle(Event::Close).await;

    assert_eq!(
        states(&session),
        [
            PopupState::Working,
            result("Hello, world."),
            PopupState::Hidden
        ]
    );
    assert_eq!(session.bridge.copied(), Vec::<String>::new());
}

#[tokio::test]
async fn closing_while_working_cancels_the_rewrite_and_a_late_answer_is_ignored() {
    let (rewriter, answer) = TestRewriter::gated();
    let session = session(FakeBridge::selecting("helo wrld"), rewriter);

    tokio::join!(session.handle(Event::ShortcutPressed), async {
        session.rewriter.called.notified().await;
        session.handle(Event::Close).await;
        let _ = answer.send(Ok("Hello, world.".to_owned()));
    });

    assert!(session.rewriter.cancelled.load(Ordering::Relaxed));
    assert_eq!(states(&session), [PopupState::Working, PopupState::Hidden]);
}

#[tokio::test]
async fn closing_a_hidden_popup_does_nothing() {
    let session = session(FakeBridge::default(), TestRewriter::returning("unused"));

    session.handle(Event::Close).await;

    assert_eq!(states(&session), []);
}

#[tokio::test]
async fn a_failed_rewrite_shows_a_message_without_any_content() {
    let session = session(
        FakeBridge::selecting("SECRET-SELECTION"),
        TestRewriter::answering(Err(RewriteError::Network)),
    );

    session.handle(Event::ShortcutPressed).await;

    let shown = states(&session);
    assert_eq!(shown.first(), Some(&PopupState::Working));
    let Some(PopupState::Message { text, .. }) = shown.get(1) else {
        panic!("expected a message, got {shown:?}");
    };
    assert!(!text.contains("SECRET-SELECTION"));
}

#[tokio::test]
async fn open_settings_hides_the_popup_and_opens_settings() {
    let session = session(
        FakeBridge::selecting("helo"),
        TestRewriter::answering(Err(RewriteError::MissingApiKey)),
    );
    session.handle(Event::ShortcutPressed).await;

    session.handle(Event::OpenSettings).await;

    assert_eq!(states(&session).last(), Some(&PopupState::Hidden));
    assert_eq!(session.popup.settings_opened.load(Ordering::Relaxed), 1);
}

#[test]
fn debug_output_hides_the_result() {
    let shown = format!("{:?}", result("SECRET-RESULT"));

    assert!(!shown.contains("SECRET-RESULT"));
}
