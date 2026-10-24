//! Update controller tests: fake ports, no network or Windows.

use std::convert::Infallible;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::sync::{Notify, oneshot};

use super::{
    AvailableUpdate, BusyGate, CheckError, FailedStep, InstallError, Notifier, UpdateController,
    UpdateEvent, UpdateSource, UpdatesState, UpdatesView,
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
    checks: AtomicUsize,
    /// The download progress an install reports, in percent.
    progress: Vec<u8>,
    /// Whether an install fails after its progress, rather than restarting
    /// the app (which never returns).
    install_fails: bool,
    /// If set, an install waits for this before it reports progress.
    install_gate: Mutex<Option<oneshot::Receiver<()>>>,
    /// The versions installs were started for.
    installs: Mutex<Vec<String>>,
    /// Notified when an install starts.
    installing: Notify,
}

impl FakeSource {
    fn answering(answer: Answer) -> Self {
        Self {
            answer,
            gate: Mutex::new(None),
            checks: AtomicUsize::new(0),
            progress: Vec::new(),
            install_fails: false,
            install_gate: Mutex::new(None),
            installs: Mutex::default(),
            installing: Notify::new(),
        }
    }
}

impl UpdateSource for FakeSource {
    type Handle = String;

    async fn check(&self) -> Result<Option<AvailableUpdate<String>>, CheckError> {
        self.checks.fetch_add(1, Ordering::Relaxed);
        let gate = self.gate.lock().unwrap().take();
        if let Some(gate) = gate {
            let _ = gate.await;
        }
        match self.answer {
            Answer::Nothing => Ok(None),
            Answer::Update(version) => Ok(Some(AvailableUpdate {
                version: version.to_owned(),
                handle: version.to_owned(),
            })),
            Answer::Fails => Err(CheckError::Network),
        }
    }

    async fn install(
        &self,
        handle: &String,
        progress: &(dyn Fn(u8) + Send + Sync),
    ) -> Result<Infallible, InstallError> {
        self.installs.lock().unwrap().push(handle.clone());
        self.installing.notify_one();
        let gate = self.install_gate.lock().unwrap().take();
        if let Some(gate) = gate {
            let _ = gate.await;
        }
        for percent in &self.progress {
            progress(*percent);
        }
        if self.install_fails {
            Err(InstallError::Signature)
        } else {
            // The real install restarts the app and never returns.
            std::future::pending().await
        }
    }
}

/// A busy gate that is idle, or busy until a test lets it go.
#[derive(Default)]
struct FakeGate {
    /// If set, the gate is busy until this fires.
    busy: Mutex<Option<oneshot::Receiver<()>>>,
    holds: AtomicUsize,
    released: Arc<AtomicUsize>,
}

/// Counts its release when dropped.
struct FakeHold(Arc<AtomicUsize>);

impl Drop for FakeHold {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }
}

impl BusyGate for FakeGate {
    type Hold = FakeHold;

    async fn hold_when_idle(&self) -> FakeHold {
        let busy = self.busy.lock().unwrap().take();
        if let Some(busy) = busy {
            let _ = busy.await;
        }
        self.holds.fetch_add(1, Ordering::Relaxed);
        FakeHold(Arc::clone(&self.released))
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

type Controller = UpdateController<FakeSource, FakeNotifier, FakeView, FakeGate>;

fn controller(answer: Answer) -> Controller {
    controller_with(FakeSource::answering(answer), FakeGate::default())
}

fn controller_with(source: FakeSource, gate: FakeGate) -> Controller {
    UpdateController::new(source, FakeNotifier::default(), FakeView::default(), gate)
}

/// A controller that already offers 1.1.0, from a source that installs it
/// with `progress` and then fails if `fails`.
async fn offering(progress: &[u8], fails: bool, gate: FakeGate) -> Controller {
    let source = FakeSource {
        progress: progress.to_vec(),
        install_fails: fails,
        ..FakeSource::answering(Answer::Update("1.1.0"))
    };
    let controller = controller_with(source, gate);
    controller.handle(UpdateEvent::CheckRequested).await;
    controller
}

/// Handles Update now until it settles. A successful install never
/// returns, so under the paused clock this gives up once nothing moves.
async fn update_now(controller: &Controller) {
    let _ = tokio::time::timeout(
        Duration::from_secs(60),
        controller.handle(UpdateEvent::UpdateNowRequested),
    )
    .await;
}

fn installs(controller: &Controller) -> Vec<String> {
    controller.source.installs.lock().unwrap().clone()
}

/// The states rendered after the update was offered.
fn states_since_offered(controller: &Controller) -> Vec<UpdatesState> {
    let states = states(controller);
    let offered = states
        .iter()
        .position(|state| *state == available("1.1.0"))
        .expect("the update was offered");
    states[offered + 1..].to_vec()
}

fn downloading(percent: u8) -> UpdatesState {
    UpdatesState::Downloading { percent }
}

fn install_failed(version: &str) -> UpdatesState {
    UpdatesState::InstallFailed {
        version: version.to_owned(),
    }
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

#[tokio::test(start_paused = true)]
async fn update_now_installs_the_offered_update_showing_its_progress() {
    let controller = offering(&[0, 12, 45, 45, 100], false, FakeGate::default()).await;

    update_now(&controller).await;

    assert_eq!(installs(&controller), ["1.1.0"]);
    assert_eq!(
        states_since_offered(&controller),
        [
            downloading(0),
            downloading(12),
            downloading(45),
            downloading(100)
        ]
    );
}

#[tokio::test(start_paused = true)]
async fn update_now_waits_until_no_rewrite_or_replace_is_running() {
    let (idle, busy) = oneshot::channel();
    let gate = FakeGate {
        busy: Mutex::new(Some(busy)),
        ..FakeGate::default()
    };
    let controller = offering(&[], false, gate).await;

    tokio::join!(update_now(&controller), async {
        tokio::task::yield_now().await;
        assert_eq!(installs(&controller), Vec::<String>::new());
        assert_eq!(states_since_offered(&controller), [downloading(0)]);
        let _ = idle.send(());
    });

    assert_eq!(controller.gate.holds.load(Ordering::Relaxed), 1);
    assert_eq!(installs(&controller), ["1.1.0"]);
}

#[tokio::test]
async fn a_failed_install_offers_the_update_again_and_lets_rewrites_run() {
    let controller = offering(&[30], true, FakeGate::default()).await;

    controller.handle(UpdateEvent::UpdateNowRequested).await;

    assert_eq!(
        states_since_offered(&controller),
        [downloading(0), downloading(30), install_failed("1.1.0")]
    );
    assert_eq!(controller.gate.released.load(Ordering::Relaxed), 1);
    assert_eq!(notifications(&controller), Vec::<String>::new());
}

#[tokio::test]
async fn update_now_can_be_tried_again_after_a_failed_install() {
    let controller = offering(&[], true, FakeGate::default()).await;
    controller.handle(UpdateEvent::UpdateNowRequested).await;

    controller.handle(UpdateEvent::UpdateNowRequested).await;

    assert_eq!(installs(&controller), ["1.1.0", "1.1.0"]);
}

#[tokio::test]
async fn nothing_starts_a_second_install_or_a_check_while_downloading() {
    let (finish, install_gate) = oneshot::channel();
    let source = FakeSource {
        install_fails: true,
        install_gate: Mutex::new(Some(install_gate)),
        ..FakeSource::answering(Answer::Update("1.1.0"))
    };
    let controller = controller_with(source, FakeGate::default());
    controller.handle(UpdateEvent::CheckRequested).await;

    tokio::join!(controller.handle(UpdateEvent::UpdateNowRequested), async {
        controller.source.installing.notified().await;
        controller.handle(UpdateEvent::UpdateNowRequested).await;
        controller.handle(UpdateEvent::CheckRequested).await;
        let _ = finish.send(());
    });

    assert_eq!(installs(&controller), ["1.1.0"]);
    assert_eq!(controller.source.checks.load(Ordering::Relaxed), 1);
    assert_eq!(
        states_since_offered(&controller),
        [downloading(0), install_failed("1.1.0")]
    );
}

#[tokio::test]
async fn update_now_does_nothing_until_an_update_is_known() {
    let controller = controller(Answer::Nothing);
    controller.handle(UpdateEvent::CheckRequested).await;

    controller.handle(UpdateEvent::UpdateNowRequested).await;

    assert_eq!(installs(&controller), Vec::<String>::new());
    assert_eq!(controller.gate.holds.load(Ordering::Relaxed), 0);
    assert_eq!(controller.state(), UpdatesState::UpToDate);
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
    assert_eq!(
        json(&downloading(45)),
        serde_json::json!({ "kind": "downloading", "percent": 45 })
    );
    assert_eq!(
        json(&install_failed("1.1.0")),
        serde_json::json!({ "kind": "installFailed", "version": "1.1.0" })
    );
}
