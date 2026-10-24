//! Update controller tests: fake ports, no network or Windows.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::sync::oneshot;
use tokio::task::JoinHandle;

use super::{
    AvailableUpdate, CheckError, FailedStep, Notifier, Remembered, UpdateController, UpdateEvent,
    UpdateMemory, UpdateSource, UpdatesState, UpdatesView,
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
    answer: Mutex<Answer>,
    /// How many checks were made.
    checks: AtomicUsize,
    /// If set, the check waits for this before it answers.
    gate: Mutex<Option<oneshot::Receiver<()>>>,
}

impl FakeSource {
    fn answering(answer: Answer) -> Self {
        Self {
            answer: Mutex::new(answer),
            checks: AtomicUsize::new(0),
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
        self.checks.fetch_add(1, Ordering::SeqCst);
        let answer = self.answer.lock().unwrap().clone();
        match answer {
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

/// Update memory kept in a field instead of the settings file.
struct FakeMemory(Mutex<Remembered>);

impl FakeMemory {
    fn new(auto_check: bool, last_notified_version: Option<&str>) -> Self {
        Self(Mutex::new(Remembered {
            auto_check,
            last_notified_version: last_notified_version.map(str::to_owned),
        }))
    }
}

impl UpdateMemory for FakeMemory {
    fn recall(&self) -> Remembered {
        self.0.lock().unwrap().clone()
    }

    fn remember_notified(&self, version: &str) {
        self.0.lock().unwrap().last_notified_version = Some(version.to_owned());
    }
}

type Controller = UpdateController<FakeSource, FakeNotifier, FakeView, FakeMemory>;

fn controller(answer: Answer) -> Controller {
    controller_remembering(answer, FakeMemory::new(true, None))
}

fn controller_remembering(answer: Answer, memory: FakeMemory) -> Controller {
    UpdateController::new(
        FakeSource::answering(answer),
        FakeNotifier::default(),
        FakeView::default(),
        memory,
    )
}

/// Starts the app's controller; its schedule runs until the test ends.
fn start(controller: &Arc<Controller>) -> JoinHandle<()> {
    let controller = Arc::clone(controller);
    tokio::spawn(async move { controller.handle(UpdateEvent::Started).await })
}

/// Lets `secs` seconds pass on the paused test clock.
async fn after(secs: u64) {
    tokio::time::sleep(Duration::from_secs(secs)).await;
}

const DAY: u64 = 24 * 60 * 60;

fn checks(controller: &Controller) -> usize {
    controller.source.checks.load(Ordering::SeqCst)
}

fn last_notified(controller: &Controller) -> Option<String> {
    controller.memory.recall().last_notified_version
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
    let controller = controller(Answer::Fails);
    controller.handle(UpdateEvent::CheckRequested).await;

    *controller.source.answer.lock().unwrap() = Answer::Update("1.1.0");
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

#[tokio::test(start_paused = true)]
async fn the_first_automatic_check_comes_10_seconds_after_start() {
    let controller = Arc::new(controller(Answer::Nothing));
    let schedule = start(&controller);

    tokio::time::sleep(Duration::from_millis(9_999)).await;
    assert_eq!(checks(&controller), 0);
    tokio::time::sleep(Duration::from_millis(2)).await;
    assert_eq!(checks(&controller), 1);

    schedule.abort();
}

#[tokio::test(start_paused = true)]
async fn automatic_checks_repeat_every_24_hours() {
    let controller = Arc::new(controller(Answer::Nothing));
    let schedule = start(&controller);

    after(11).await;
    after(DAY - 2).await;
    assert_eq!(checks(&controller), 1);
    after(2).await;
    assert_eq!(checks(&controller), 2);
    after(DAY).await;
    assert_eq!(checks(&controller), 3);

    schedule.abort();
}

#[tokio::test(start_paused = true)]
async fn nothing_is_checked_while_automatic_checks_are_off() {
    let controller = Arc::new(controller_remembering(
        Answer::Update("1.1.0"),
        FakeMemory::new(false, None),
    ));
    let schedule = start(&controller);

    after(3 * DAY).await;

    assert_eq!(checks(&controller), 0);
    assert_eq!(states(&controller), Vec::<UpdatesState>::new());
    schedule.abort();
}

#[tokio::test(start_paused = true)]
async fn turning_automatic_checks_off_stops_the_schedule_and_on_restarts_it() {
    let controller = Arc::new(controller(Answer::Nothing));
    let schedule = start(&controller);
    after(5).await;

    controller
        .handle(UpdateEvent::AutoCheckChanged(false))
        .await;
    after(3 * DAY).await;
    assert_eq!(checks(&controller), 0);

    controller.handle(UpdateEvent::AutoCheckChanged(true)).await;
    after(9).await;
    assert_eq!(checks(&controller), 0);
    after(2).await;
    assert_eq!(checks(&controller), 1);

    schedule.abort();
}

#[tokio::test]
async fn a_manual_check_still_works_with_automatic_checks_off() {
    let controller = controller_remembering(Answer::Update("1.1.0"), FakeMemory::new(false, None));

    controller.handle(UpdateEvent::CheckRequested).await;

    assert_eq!(controller.state(), available("1.1.0"));
}

#[tokio::test(start_paused = true)]
async fn an_automatic_check_that_finds_an_update_notifies_and_offers_it() {
    let controller = Arc::new(controller(Answer::Update("1.1.0")));
    let schedule = start(&controller);

    after(11).await;

    // No "Checking…" flashes up for a check the user didn't ask for.
    assert_eq!(states(&controller), [available("1.1.0")]);
    assert_eq!(notifications(&controller), ["1.1.0"]);
    assert_eq!(last_notified(&controller).as_deref(), Some("1.1.0"));
    schedule.abort();
}

#[tokio::test(start_paused = true)]
async fn each_version_is_announced_only_once() {
    let controller = Arc::new(controller(Answer::Update("1.1.0")));
    let schedule = start(&controller);

    after(11 + 3 * DAY).await;

    assert_eq!(checks(&controller), 4);
    assert_eq!(notifications(&controller), ["1.1.0"]);
    schedule.abort();
}

#[tokio::test(start_paused = true)]
async fn a_version_announced_in_an_earlier_run_is_not_announced_again() {
    let controller = Arc::new(controller_remembering(
        Answer::Update("1.1.0"),
        FakeMemory::new(true, Some("1.1.0")),
    ));
    let schedule = start(&controller);

    after(11).await;

    assert_eq!(notifications(&controller), Vec::<String>::new());
    // The tray still offers it.
    assert_eq!(controller.state(), available("1.1.0"));
    schedule.abort();
}

#[tokio::test(start_paused = true)]
async fn an_even_newer_version_is_announced_too() {
    let controller = Arc::new(controller(Answer::Update("1.1.0")));
    let schedule = start(&controller);
    after(11).await;

    *controller.source.answer.lock().unwrap() = Answer::Update("1.2.0");
    after(DAY).await;

    assert_eq!(notifications(&controller), ["1.1.0", "1.2.0"]);
    assert_eq!(last_notified(&controller).as_deref(), Some("1.2.0"));
    assert_eq!(controller.state(), available("1.2.0"));
    schedule.abort();
}

#[tokio::test(start_paused = true)]
async fn versions_are_compared_as_versions_not_text() {
    let controller = Arc::new(controller_remembering(
        Answer::Update("1.10.0"),
        FakeMemory::new(true, Some("1.9.0")),
    ));
    let schedule = start(&controller);
    after(11).await;

    *controller.source.answer.lock().unwrap() = Answer::Update("1.2.0");
    after(DAY).await;

    assert_eq!(notifications(&controller), ["1.10.0"]);
    assert_eq!(last_notified(&controller).as_deref(), Some("1.10.0"));
    schedule.abort();
}

#[tokio::test(start_paused = true)]
async fn a_failed_automatic_check_shows_nothing() {
    let controller = Arc::new(controller(Answer::Fails));
    let schedule = start(&controller);

    after(11).await;

    assert_eq!(checks(&controller), 1);
    assert_eq!(states(&controller), Vec::<UpdatesState>::new());
    assert_eq!(controller.state(), UpdatesState::Idle);
    assert_eq!(notifications(&controller), Vec::<String>::new());
    schedule.abort();
}

#[tokio::test(start_paused = true)]
async fn a_failed_automatic_check_keeps_the_update_already_found() {
    let controller = Arc::new(controller(Answer::Update("1.1.0")));
    let schedule = start(&controller);
    after(11).await;

    *controller.source.answer.lock().unwrap() = Answer::Fails;
    after(DAY).await;

    assert_eq!(checks(&controller), 2);
    assert_eq!(states(&controller), [available("1.1.0")]);
    schedule.abort();
}

#[tokio::test(start_paused = true)]
async fn turning_automatic_checks_off_keeps_the_known_update() {
    let controller = Arc::new(controller(Answer::Update("1.1.0")));
    let schedule = start(&controller);
    after(11).await;

    controller
        .handle(UpdateEvent::AutoCheckChanged(false))
        .await;
    after(DAY).await;

    assert_eq!(controller.state(), available("1.1.0"));
    assert_eq!(states(&controller), [available("1.1.0")]);
    schedule.abort();
}
