//! Rewrite session tests: fake ports, no Windows, network or key.

use std::future::{Future, ready};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;

use tokio::sync::{Notify, oneshot};

use super::{
    BridgeError, Captured, Event, MAX_SELECTION_CHARS, MessageAction, PopupState, PopupView,
    REWRITE_TIMEOUT, RewriteSession, TextBridge,
};
use crate::llm::{InvalidSetting, RewriteError, Rewriter};
use crate::settings::{ModelSettings, Provider, ProviderReadiness, SettingsView};

/// A bridge whose capture finds `selection`, recording what it is asked
/// to do.
#[derive(Default)]
struct FakeBridge {
    selection: Option<String>,
    copy_fails: bool,
    window_closed: bool,
    replace_fails: bool,
    captures: AtomicUsize,
    copied: Mutex<Vec<String>>,
    replaced: Mutex<Vec<(u32, String)>>,
    /// If set, Replace waits for this before it finishes.
    replace_gate: Mutex<Option<oneshot::Receiver<()>>>,
    /// Notified when Replace starts.
    replacing: Notify,
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

    fn replaced(&self) -> Vec<(u32, String)> {
        self.replaced.lock().unwrap().clone()
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

    fn is_open(&self, _window: &u32) -> bool {
        !self.window_closed
    }

    async fn replace(&self, window: &u32, text: &str) -> Result<(), BridgeError> {
        let gate = self.replace_gate.lock().unwrap().take();
        self.replacing.notify_one();
        if let Some(gate) = gate {
            let _ = gate.await;
        }
        if self.replace_fails {
            return Err(BridgeError);
        }
        self.replaced
            .lock()
            .unwrap()
            .push((*window, text.to_owned()));
        Ok(())
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

    /// Like [`TestRewriter::gated`], but later calls answer `then` at once.
    fn gated_then(
        then: Result<String, RewriteError>,
    ) -> (Self, oneshot::Sender<Result<String, RewriteError>>) {
        let (rewriter, answer) = Self::gated();
        let rewriter = Self {
            answer: Some(then),
            ..rewriter
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

/// Settings whose readiness and active instructions a test can set.
struct FakeSettings {
    readiness: ProviderReadiness,
    instructions: Mutex<String>,
}

impl Default for FakeSettings {
    fn default() -> Self {
        Self {
            readiness: ProviderReadiness::Ready,
            instructions: Mutex::new(INSTRUCTIONS.to_owned()),
        }
    }
}

impl FakeSettings {
    fn not_ready(readiness: ProviderReadiness) -> Self {
        Self {
            readiness,
            ..Self::default()
        }
    }

    fn activate(&self, instructions: &str) {
        *self.instructions.lock().unwrap() = instructions.to_owned();
    }
}

impl SettingsView for FakeSettings {
    fn model_settings(&self) -> ModelSettings {
        ModelSettings {
            provider: Provider::OpenAi,
            model: "gpt-test".to_owned(),
        }
    }

    fn provider_readiness(&self) -> ProviderReadiness {
        self.readiness
    }

    fn active_instructions(&self) -> String {
        self.instructions.lock().unwrap().clone()
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
    session_with(bridge, rewriter, FakeSettings::default())
}

fn session_with(bridge: FakeBridge, rewriter: TestRewriter, settings: FakeSettings) -> TestSession {
    RewriteSession::new(bridge, rewriter, settings, RecordingPopup::default())
}

fn message(text: &str, action: Option<MessageAction>) -> PopupState {
    PopupState::Message {
        text: text.to_owned(),
        action,
        copied: false,
        rewrite: false,
    }
}

/// The message for a failed rewrite, which offers Rewrite.
fn failed(text: &str) -> PopupState {
    PopupState::Message {
        text: text.to_owned(),
        action: None,
        copied: false,
        rewrite: true,
    }
}

fn states(session: &TestSession) -> Vec<PopupState> {
    session.popup.states.lock().unwrap().clone()
}

fn working(text: &str) -> PopupState {
    PopupState::Working {
        text: text.to_owned(),
    }
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
        [working("helo wrld"), result("Hello, world.")]
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
        [working("Hello, world."), result("Hello, world.")]
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
        [working("helo wrld"), result("Hello, world.")]
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
async fn replace_puts_the_result_into_the_original_window_and_hides_the_popup() {
    let session = rewritten("helo wrld", "Hello, world.").await;

    session.handle(Event::Replace).await;

    assert_eq!(session.bridge.replaced(), [(7, "Hello, world.".to_owned())]);
    assert_eq!(
        states(&session),
        [
            working("helo wrld"),
            result("Hello, world."),
            PopupState::Hidden
        ]
    );
    assert_eq!(session.bridge.copied(), Vec::<String>::new());
}

#[tokio::test]
async fn replace_into_a_closed_window_copies_the_result_and_shows_a_message() {
    let session = session(
        FakeBridge {
            window_closed: true,
            ..FakeBridge::selecting("helo wrld")
        },
        TestRewriter::returning("Hello, world."),
    );
    session.handle(Event::ShortcutPressed).await;

    session.handle(Event::Replace).await;

    assert_eq!(session.bridge.replaced(), []);
    assert_eq!(session.bridge.copied(), ["Hello, world."]);
    assert_eq!(
        states(&session).last(),
        Some(&PopupState::Message {
            text: "The original window closed. Press Ctrl + V to paste the rewrite.".to_owned(),
            action: None,
            copied: true,
            rewrite: false,
        })
    );
}

#[tokio::test]
async fn a_failed_replace_copies_the_result_and_says_so() {
    let session = session(
        FakeBridge {
            replace_fails: true,
            ..FakeBridge::selecting("helo wrld")
        },
        TestRewriter::returning("Hello, world."),
    );
    session.handle(Event::ShortcutPressed).await;

    session.handle(Event::Replace).await;

    assert_eq!(session.bridge.copied(), ["Hello, world."]);
    assert_eq!(
        states(&session).last(),
        Some(&PopupState::Message {
            text: "Couldn't replace your text. Press Ctrl + V to paste the rewrite.".to_owned(),
            action: None,
            copied: true,
            rewrite: false,
        })
    );
}

#[tokio::test]
async fn replace_into_a_closed_window_keeps_the_result_shown_if_copy_fails() {
    let session = session(
        FakeBridge {
            window_closed: true,
            copy_fails: true,
            ..FakeBridge::selecting("helo wrld")
        },
        TestRewriter::returning("Hello, world."),
    );
    session.handle(Event::ShortcutPressed).await;

    session.handle(Event::Replace).await;

    assert_eq!(states(&session).last(), Some(&result("Hello, world.")));
}

#[tokio::test]
async fn replace_does_nothing_while_working() {
    let (rewriter, _answer) = TestRewriter::gated();
    let session = session(FakeBridge::selecting("helo wrld"), rewriter);

    tokio::join!(session.handle(Event::ShortcutPressed), async {
        session.rewriter.called.notified().await;
        session.handle(Event::Replace).await;
        session.handle(Event::Close).await;
    });

    assert_eq!(session.bridge.replaced(), []);
    assert_eq!(states(&session), [working("helo wrld"), PopupState::Hidden]);
}

#[tokio::test]
async fn the_shortcut_does_not_capture_while_replacing() {
    let (finish_replace, gate) = oneshot::channel();
    let session = session(
        FakeBridge {
            replace_gate: Mutex::new(Some(gate)),
            ..FakeBridge::selecting("helo wrld")
        },
        TestRewriter::returning("Hello, world."),
    );
    session.handle(Event::ShortcutPressed).await;

    tokio::join!(session.handle(Event::Replace), async {
        session.bridge.replacing.notified().await;
        // Hiding the popup makes it lose focus, which closes it.
        session.handle(Event::Close).await;
        session.handle(Event::ShortcutPressed).await;
        let _ = finish_replace.send(());
    });

    assert_eq!(session.bridge.captures.load(Ordering::Relaxed), 1);
    assert_eq!(session.bridge.replaced(), [(7, "Hello, world.".to_owned())]);
}

#[tokio::test]
async fn close_hides_the_result() {
    let session = rewritten("helo wrld", "Hello, world.").await;

    session.handle(Event::Close).await;

    assert_eq!(
        states(&session),
        [
            working("helo wrld"),
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
    assert_eq!(states(&session), [working("helo wrld"), PopupState::Hidden]);
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
    assert_eq!(shown.first(), Some(&working("SECRET-SELECTION")));
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
fn the_limits_are_5000_characters_and_30_seconds() {
    assert_eq!(MAX_SELECTION_CHARS, 5_000);
    assert_eq!(REWRITE_TIMEOUT, Duration::from_secs(30));
}

#[tokio::test]
async fn a_selection_over_5000_characters_shows_too_long_and_sends_nothing() {
    let session = session(
        FakeBridge::selecting(&"a".repeat(5_001)),
        TestRewriter::returning("unused"),
    );

    session.handle(Event::ShortcutPressed).await;

    assert_eq!(
        states(&session),
        [message(
            "This text is too long. Select up to 5,000 characters.",
            None
        )]
    );
    assert_eq!(session.rewriter.calls(), []);
}

#[tokio::test]
async fn a_selection_of_5000_characters_is_rewritten() {
    // 5,000 characters but 10,000 bytes: the limit counts characters.
    let session = rewritten(&"é".repeat(5_000), "done").await;

    assert_eq!(
        states(&session),
        [working(&"é".repeat(5_000)), result("done")]
    );
}

#[tokio::test]
async fn no_key_asks_for_one_with_open_settings_and_sends_nothing() {
    let session = session_with(
        FakeBridge::selecting("helo"),
        TestRewriter::returning("unused"),
        FakeSettings::not_ready(ProviderReadiness::MissingKey),
    );

    session.handle(Event::ShortcutPressed).await;

    assert_eq!(
        states(&session),
        [message(
            "Add your API key to get started.",
            Some(MessageAction::OpenSettings)
        )]
    );
    assert_eq!(session.rewriter.calls(), []);
}

#[tokio::test]
async fn no_model_asks_for_one_with_open_settings_and_sends_nothing() {
    let session = session_with(
        FakeBridge::selecting("helo"),
        TestRewriter::returning("unused"),
        FakeSettings::not_ready(ProviderReadiness::NoModel),
    );

    session.handle(Event::ShortcutPressed).await;

    assert_eq!(
        states(&session),
        [message(
            "Choose a model to get started.",
            Some(MessageAction::OpenSettings)
        )]
    );
    assert_eq!(session.rewriter.calls(), []);
}

#[tokio::test]
async fn no_custom_base_url_asks_for_one_with_open_settings_and_sends_nothing() {
    let session = session_with(
        FakeBridge::selecting("helo"),
        TestRewriter::returning("unused"),
        FakeSettings::not_ready(ProviderReadiness::NoCustomUrl),
    );

    session.handle(Event::ShortcutPressed).await;

    assert_eq!(
        states(&session),
        [message(
            "Add your base URL to get started.",
            Some(MessageAction::OpenSettings)
        )]
    );
    assert_eq!(session.rewriter.calls(), []);
}

#[tokio::test]
async fn open_settings_from_the_no_key_message_closes_the_popup() {
    let session = session_with(
        FakeBridge::selecting("helo"),
        TestRewriter::returning("unused"),
        FakeSettings::not_ready(ProviderReadiness::MissingKey),
    );
    session.handle(Event::ShortcutPressed).await;

    session.handle(Event::OpenSettings).await;

    assert_eq!(states(&session).last(), Some(&PopupState::Hidden));
    assert_eq!(session.popup.settings_opened.load(Ordering::Relaxed), 1);
}

#[tokio::test(start_paused = true)]
async fn a_rewrite_with_no_answer_after_30_seconds_is_cancelled() {
    let (rewriter, _answer) = TestRewriter::gated();
    let session = session(FakeBridge::selecting("helo wrld"), rewriter);

    tokio::join!(session.handle(Event::ShortcutPressed), async {
        session.rewriter.called.notified().await;
        tokio::time::sleep(Duration::from_millis(29_999)).await;
        assert_eq!(states(&session), [working("helo wrld")]);
    });

    assert!(session.rewriter.cancelled.load(Ordering::Relaxed));
    assert_eq!(
        states(&session),
        [
            working("helo wrld"),
            failed("OpenAI took too long. Try again.")
        ]
    );
}

#[tokio::test]
async fn the_shortcut_while_working_does_nothing() {
    let (rewriter, first_answer) = TestRewriter::gated_then(Ok("Second.".to_owned()));
    let session = session(FakeBridge::selecting("helo wrld"), rewriter);

    tokio::join!(session.handle(Event::ShortcutPressed), async {
        session.rewriter.called.notified().await;
        session.handle(Event::ShortcutPressed).await;
        let _ = first_answer.send(Ok("First.".to_owned()));
    });

    assert_eq!(session.bridge.captures.load(Ordering::Relaxed), 1);
    assert!(!session.rewriter.cancelled.load(Ordering::Relaxed));
    assert_eq!(session.rewriter.calls().len(), 1);
    assert_eq!(states(&session), [working("helo wrld"), result("First.")]);
}

#[tokio::test]
async fn the_shortcut_on_a_failed_rewrite_tries_the_same_selection_again() {
    let (rewriter, first_answer) = TestRewriter::gated_then(Ok("Hello, world.".to_owned()));
    let session = session(FakeBridge::selecting("helo wrld"), rewriter);
    tokio::join!(session.handle(Event::ShortcutPressed), async {
        session.rewriter.called.notified().await;
        let _ = first_answer.send(Err(RewriteError::RateLimited));
    });
    session.settings.activate("Be formal.");

    session.handle(Event::ShortcutPressed).await;

    assert_eq!(session.bridge.captures.load(Ordering::Relaxed), 1);
    assert_eq!(
        session.rewriter.calls(),
        [
            (INSTRUCTIONS.to_owned(), "helo wrld".to_owned()),
            ("Be formal.".to_owned(), "helo wrld".to_owned()),
        ]
    );
    assert_eq!(states(&session).last(), Some(&result("Hello, world.")));
}

#[tokio::test]
async fn the_shortcut_on_a_result_rewrites_the_same_selection_again() {
    let session = rewritten("helo wrld", "Hello, world.").await;

    session.handle(Event::ShortcutPressed).await;

    assert_eq!(session.bridge.captures.load(Ordering::Relaxed), 1);
    assert_eq!(session.rewriter.calls().len(), 2);
    assert_eq!(
        states(&session),
        [
            working("helo wrld"),
            result("Hello, world."),
            working("helo wrld"),
            result("Hello, world."),
        ]
    );
}

#[tokio::test]
async fn each_rewrite_error_shows_its_own_message() {
    let open_settings = Some(MessageAction::OpenSettings);
    let cases = [
        (
            RewriteError::Network,
            failed("Couldn't reach OpenAI. Check your connection and try again."),
        ),
        (
            RewriteError::Unauthorized,
            message(
                "OpenAI rejected your API key. Check it in Settings.",
                open_settings,
            ),
        ),
        (
            RewriteError::RateLimited,
            failed("OpenAI is busy. Wait a moment and try again."),
        ),
        (
            RewriteError::Provider { status: 500 },
            failed("OpenAI returned an error. Try again."),
        ),
        (
            RewriteError::EmptyResponse,
            failed("OpenAI sent back nothing. Try again."),
        ),
        (
            RewriteError::Other,
            failed("Couldn't rewrite your text. Try again."),
        ),
        (
            RewriteError::MissingApiKey,
            message("Add your API key to get started.", open_settings),
        ),
        (
            RewriteError::KeyStore,
            message(
                "Couldn't read your saved API key. Enter it again in Settings.",
                open_settings,
            ),
        ),
        (
            RewriteError::InvalidSettings(InvalidSetting::Model),
            message("Choose a model to get started.", open_settings),
        ),
        (
            RewriteError::InvalidSettings(InvalidSetting::BaseUrl),
            message(
                "Enter a base URL starting with http:// or https://.",
                open_settings,
            ),
        ),
    ];

    for (error, shown) in cases {
        let session = session(
            FakeBridge::selecting("helo"),
            TestRewriter::answering(Err(error.clone())),
        );

        session.handle(Event::ShortcutPressed).await;

        assert_eq!(states(&session), [working("helo"), shown], "for {error:?}");
    }
}

#[test]
fn debug_output_hides_the_result() {
    let shown = format!("{:?}", result("SECRET-RESULT"));

    assert!(!shown.contains("SECRET-RESULT"));
}

// The busy gate: an update waits for the session to be idle and holds it.

#[tokio::test]
async fn an_idle_session_is_held_at_once_and_then_ignores_the_shortcut() {
    let session = session(
        FakeBridge::selecting("helo wrld"),
        TestRewriter::returning("Hello, world."),
    );

    session.hold_when_idle().await;
    session.handle(Event::ShortcutPressed).await;

    assert_eq!(session.bridge.captures.load(Ordering::Relaxed), 0);
    assert_eq!(states(&session), []);
}

#[tokio::test]
async fn holding_waits_for_a_running_rewrite_then_closes_its_result() {
    let (rewriter, answer) = TestRewriter::gated();
    let session = session(FakeBridge::selecting("helo wrld"), rewriter);

    tokio::join!(session.handle(Event::ShortcutPressed), async {
        session.rewriter.called.notified().await;
        tokio::join!(
            async {
                session.hold_when_idle().await;
                assert_eq!(states(&session)[1], result("Hello, world."));
            },
            async {
                tokio::task::yield_now().await;
                let _ = answer.send(Ok("Hello, world.".to_owned()));
            }
        );
    });

    assert!(!session.rewriter.cancelled.load(Ordering::Relaxed));
    assert_eq!(
        states(&session),
        [
            working("helo wrld"),
            result("Hello, world."),
            PopupState::Hidden
        ]
    );
}

#[tokio::test]
async fn holding_waits_for_a_running_replace() {
    let (finish_replace, gate) = oneshot::channel();
    let session = session(
        FakeBridge {
            replace_gate: Mutex::new(Some(gate)),
            ..FakeBridge::selecting("helo wrld")
        },
        TestRewriter::returning("Hello, world."),
    );
    session.handle(Event::ShortcutPressed).await;

    tokio::join!(session.handle(Event::Replace), async {
        session.bridge.replacing.notified().await;
        tokio::join!(
            async {
                session.hold_when_idle().await;
                assert_eq!(session.bridge.replaced(), [(7, "Hello, world.".to_owned())]);
            },
            async {
                tokio::task::yield_now().await;
                let _ = finish_replace.send(());
            }
        );
    });
}

#[tokio::test]
async fn holding_closes_a_message() {
    let session = session_with(
        FakeBridge::selecting("helo wrld"),
        TestRewriter::returning("unused"),
        FakeSettings::not_ready(ProviderReadiness::MissingKey),
    );
    session.handle(Event::ShortcutPressed).await;

    session.hold_when_idle().await;

    assert_eq!(states(&session).last(), Some(&PopupState::Hidden));
}

#[tokio::test]
async fn a_released_session_rewrites_again() {
    let session = session(
        FakeBridge::selecting("helo wrld"),
        TestRewriter::returning("Hello, world."),
    );
    session.hold_when_idle().await;

    session.release();
    session.handle(Event::ShortcutPressed).await;

    assert_eq!(
        states(&session),
        [working("helo wrld"), result("Hello, world.")]
    );
}

// Rewrite: a new rewrite of the same selection, from the popup.

#[tokio::test]
async fn rewrite_on_a_result_rewrites_the_same_selection_with_the_active_instructions() {
    let session = rewritten("helo wrld", "Hello, world.").await;
    session.settings.activate("Be formal.");

    session.handle(Event::Rewrite).await;

    assert_eq!(session.bridge.captures.load(Ordering::Relaxed), 1);
    assert_eq!(
        session.rewriter.calls(),
        [
            (INSTRUCTIONS.to_owned(), "helo wrld".to_owned()),
            ("Be formal.".to_owned(), "helo wrld".to_owned()),
        ]
    );
    assert_eq!(
        states(&session),
        [
            working("helo wrld"),
            result("Hello, world."),
            working("helo wrld"),
            result("Hello, world."),
        ]
    );
}

#[tokio::test]
async fn rewrite_on_a_failed_rewrite_tries_the_same_selection_again() {
    let (rewriter, first_answer) = TestRewriter::gated_then(Ok("Hello, world.".to_owned()));
    let session = session(FakeBridge::selecting("helo wrld"), rewriter);
    tokio::join!(session.handle(Event::ShortcutPressed), async {
        session.rewriter.called.notified().await;
        let _ = first_answer.send(Err(RewriteError::Network));
    });

    session.handle(Event::Rewrite).await;

    assert_eq!(session.bridge.captures.load(Ordering::Relaxed), 1);
    assert_eq!(
        states(&session),
        [
            working("helo wrld"),
            failed("Couldn't reach OpenAI. Check your connection and try again."),
            working("helo wrld"),
            result("Hello, world."),
        ]
    );
}

#[tokio::test]
async fn rewrite_does_nothing_while_working() {
    let (rewriter, answer) = TestRewriter::gated();
    let session = session(FakeBridge::selecting("helo wrld"), rewriter);

    tokio::join!(session.handle(Event::ShortcutPressed), async {
        session.rewriter.called.notified().await;
        session.handle(Event::Rewrite).await;
        let _ = answer.send(Ok("Hello, world.".to_owned()));
    });

    assert!(!session.rewriter.cancelled.load(Ordering::Relaxed));
    assert_eq!(session.rewriter.calls().len(), 1);
    assert_eq!(
        states(&session),
        [working("helo wrld"), result("Hello, world.")]
    );
}

#[tokio::test]
async fn rewrite_does_nothing_while_the_popup_is_hidden() {
    let session = session(
        FakeBridge::selecting("helo wrld"),
        TestRewriter::returning("unused"),
    );

    session.handle(Event::Rewrite).await;

    assert_eq!(session.bridge.captures.load(Ordering::Relaxed), 0);
    assert_eq!(session.rewriter.calls(), []);
    assert_eq!(states(&session), []);
}

#[tokio::test]
async fn rewrite_after_close_does_nothing() {
    let session = rewritten("helo wrld", "Hello, world.").await;
    session.handle(Event::Close).await;

    session.handle(Event::Rewrite).await;

    assert_eq!(session.rewriter.calls().len(), 1);
    assert_eq!(states(&session).last(), Some(&PopupState::Hidden));
}

#[tokio::test]
async fn rewrite_does_nothing_while_replacing() {
    let (finish_replace, gate) = oneshot::channel();
    let session = session(
        FakeBridge {
            replace_gate: Mutex::new(Some(gate)),
            ..FakeBridge::selecting("helo wrld")
        },
        TestRewriter::returning("Hello, world."),
    );
    session.handle(Event::ShortcutPressed).await;

    tokio::join!(session.handle(Event::Replace), async {
        session.bridge.replacing.notified().await;
        session.handle(Event::Rewrite).await;
        let _ = finish_replace.send(());
    });

    assert_eq!(session.rewriter.calls().len(), 1);
    assert_eq!(session.bridge.replaced(), [(7, "Hello, world.".to_owned())]);
    assert_eq!(
        states(&session),
        [
            working("helo wrld"),
            result("Hello, world."),
            PopupState::Hidden
        ]
    );
}
