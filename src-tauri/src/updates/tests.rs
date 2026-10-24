//! Update controller tests: fake ports, no network or Windows.

use std::sync::Mutex;

use tokio::sync::oneshot;

use super::{
    AvailableUpdate, CheckError, FailedStep, Notifier, UpdateController, UpdateEvent, UpdateSource,
    UpdatesState, UpdatesView,
};

/// What the fake source answers to a check.
#[derive(Clone)]
enum Answer {
    Nothing,
    Update(&'static str),
    Fails,
}

/// An update source that gives the scripted answer, optionally only once a
/// test lets it.
struct FakeSource {
    answer: Answer,
    /// If set, the check waits for this before it answers.
    gate: Mutex<Option<oneshot::Receiver<()>>>,
}

impl FakeSource {
    fn answering(answer: Answer) -> Self {
        Self {
            answer,
            gate: Mutex::new(None),
        }
    }
}

impl UpdateSource for FakeSource {
    type Handle = ();

    async fn check(&self) -> Result<Option<AvailableUpdate<()>>, CheckError> {
        let gate = self.gate.lock().unwrap().take();
        if let Some(gate) = gate {
            let _ = gate.await;
        }
        match self.answer {
            Answer::Nothing => Ok(None),
            Answer::Update(version) => Ok(Some(AvailableUpdate {
                version: version.to_owned(),
                handle: (),
            })),
            Answer::Fails => Err(CheckError::Network),
        }
    }
}

/// Records every Windows notification asked for.
#[derive(Default)]
struct FakeNotifier {
    shown: Mutex<Vec<String>>,
}

impl Notifier for FakeNotifier {
    fn update_available(&self, version: &str) {
        self.shown.lock().unwrap().push(version.to_owned());
    }
}

/// Records every state rendered.
#[derive(Default)]
struct FakeView {
    states: Mutex<Vec<UpdatesState>>,
}

impl UpdatesView for FakeView {
    fn render(&self, state: &UpdatesState) {
        self.states.lock().unwrap().push(state.clone());
    }
}

type Controller = UpdateController<FakeSource, FakeNotifier, FakeView>;

fn controller(answer: Answer) -> Controller {
    UpdateController::new(
        FakeSource::answering(answer),
        FakeNotifier::default(),
        FakeView::default(),
    )
}

fn states(controller: &Controller) -> Vec<UpdatesState> {
    controller.view.states.lock().unwrap().clone()
}

fn notifications(controller: &Controller) -> Vec<String> {
    controller.notifier.shown.lock().unwrap().clone()
}

fn available(version: &str) -> UpdatesState {
    UpdatesState::Available {
        version: version.to_owned(),
    }
}

#[tokio::test]
async fn a_check_that_finds_nothing_says_up_to_date() {
    let controller = controller(Answer::Nothing);

    controller.handle(UpdateEvent::CheckRequested).await;

    assert_eq!(
        states(&controller),
        [UpdatesState::Checking, UpdatesState::UpToDate]
    );
}

#[tokio::test]
async fn a_check_that_finds_a_newer_version_offers_it() {
    let controller = controller(Answer::Update("1.1.0"));

    controller.handle(UpdateEvent::CheckRequested).await;

    assert_eq!(
        states(&controller),
        [UpdatesState::Checking, available("1.1.0")]
    );
}

#[tokio::test]
async fn a_manual_check_never_notifies() {
    let controller = controller(Answer::Update("1.1.0"));

    controller.handle(UpdateEvent::CheckRequested).await;
    controller.handle(UpdateEvent::CheckRequested).await;

    assert_eq!(notifications(&controller), Vec::<String>::new());
}

#[tokio::test]
async fn a_failed_manual_check_says_so() {
    let controller = controller(Answer::Fails);

    controller.handle(UpdateEvent::CheckRequested).await;

    assert_eq!(
        states(&controller),
        [
            UpdatesState::Checking,
            UpdatesState::Failed {
                step: FailedStep::Check
            }
        ]
    );
}

#[tokio::test]
async fn checking_again_after_a_failure_can_find_the_update() {
    let mut controller = controller(Answer::Fails);
    controller.handle(UpdateEvent::CheckRequested).await;

    controller.source.answer = Answer::Update("1.1.0");
    controller.handle(UpdateEvent::CheckRequested).await;

    assert_eq!(controller.state(), available("1.1.0"));
}

#[tokio::test]
async fn a_check_requested_while_one_runs_waits_for_that_answer() {
    let controller = controller(Answer::Nothing);
    let (answer, gate) = oneshot::channel();
    *controller.source.gate.lock().unwrap() = Some(gate);

    tokio::join!(controller.handle(UpdateEvent::CheckRequested), async {
        controller.handle(UpdateEvent::CheckRequested).await;
        let _ = answer.send(());
    });

    assert_eq!(
        states(&controller),
        [UpdatesState::Checking, UpdatesState::UpToDate]
    );
}

#[test]
fn the_state_reaches_the_settings_page_tagged_by_kind() {
    let json = |state: &UpdatesState| serde_json::to_value(state).unwrap();

    assert_eq!(
        json(&UpdatesState::Idle),
        serde_json::json!({ "kind": "idle" })
    );
    assert_eq!(
        json(&UpdatesState::UpToDate),
        serde_json::json!({ "kind": "upToDate" })
    );
    assert_eq!(
        json(&available("1.1.0")),
        serde_json::json!({ "kind": "available", "version": "1.1.0" })
    );
    assert_eq!(
        json(&UpdatesState::Failed {
            step: FailedStep::Check
        }),
        serde_json::json!({ "kind": "failed", "step": "check" })
    );
}
