//! The update controller: the one module that knows the update rules.
//!
//! Like the rewrite session, it reacts to [`UpdateEvent`]s fed to
//! [`UpdateController::handle`] and reaches the outside only through ports,
//! so tests drive it with fakes and no network or Windows:
//!
//! - [`UpdateSource`]: asks GitHub whether a newer version exists;
//! - [`Notifier`]: the Windows notifications about updates;
//! - [`UpdatesView`]: shows each [`UpdatesState`] in Settings and the tray;
//! - [`BusyGate`]: lets an install wait until no rewrite or replace runs;
//! - [`UpdateMemory`]: what is kept between runs, in the settings file.
//!
//! The controller owns the automatic-check schedule: the first check
//! [`FIRST_CHECK_DELAY`] after the app starts, then every
//! [`CHECK_INTERVAL`], only while automatic checks are on.
//!
//! Privacy: a check sends nothing about the user, and failures are logged
//! only as a fixed error kind, never with URLs or response bodies.

use std::convert::Infallible;
use std::future::{Future, poll_fn};
use std::pin::pin;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::task::Poll;
use std::time::Duration;

use serde::Serialize;
use tokio::sync::watch;

/// How long after the app starts the first automatic check waits, so
/// starting with Windows stays fast.
pub const FIRST_CHECK_DELAY: Duration = Duration::from_secs(10);

/// How often automatic checks repeat while the app runs.
pub const CHECK_INTERVAL: Duration = Duration::from_hours(24);

/// Something that asks the controller to act.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateEvent {
    /// The app launched. Handling it runs the automatic-check schedule for
    /// as long as the app runs, so it never returns: send it once, on its
    /// own task.
    Started,
    /// The user asked for a check: the tray item or the Settings button.
    CheckRequested,
    /// The user turned automatic checks on or off. The choice is already
    /// saved; this only starts or stops the schedule.
    AutoCheckChanged(bool),
    /// The user clicked Update now in Settings. The only way to install.
    UpdateNowRequested,
}

/// What Settings and the tray show about updates. Serialized for the
/// Settings page as `{ "kind": "checking" }`,
/// `{ "kind": "available", "version": "1.1.0" }` and so on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum UpdatesState {
    /// Nothing is known yet.
    Idle,
    /// A check is running.
    Checking,
    /// The last check found nothing newer.
    UpToDate,
    /// A newer version can be installed.
    Available {
        /// The newer version, such as `1.1.0`.
        version: String,
    },
    /// The last attempt failed.
    Failed {
        /// What failed.
        step: FailedStep,
    },
    /// The update is downloading; the app restarts once it is installed.
    Downloading {
        /// How much is downloaded, from 0 to 100.
        percent: u8,
    },
    /// Installing this version failed. It is still offered, and the old
    /// version keeps running.
    InstallFailed {
        /// The version that couldn't be installed, such as `1.1.0`.
        version: String,
    },
}

/// The step of updating that failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FailedStep {
    /// Asking whether there is an update.
    Check,
}

/// A newer version found by a check.
pub struct AvailableUpdate<H> {
    /// The newer version, such as `1.1.0`.
    pub version: String,
    /// What the source needs to install it. Opaque to the controller.
    pub handle: H,
}

/// Why a check failed. Holds no URL or response content, so it is safe to
/// log.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum CheckError {
    /// GitHub couldn't be reached or answered with an error.
    #[error("network")]
    Network,
    /// The check took too long.
    #[error("timeout")]
    Timeout,
    /// The update file couldn't be understood.
    #[error("bad response")]
    BadResponse,
    /// Anything else, such as the updater being unavailable.
    #[error("other")]
    Other,
}

/// Why an install failed. Holds no URL or response content, so it is safe
/// to log.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InstallError {
    /// The download couldn't be finished.
    #[error("network")]
    Network,
    /// The download wasn't signed with the project's key.
    #[error("signature")]
    Signature,
    /// Anything else, such as the installer not starting.
    #[error("other")]
    Other,
}

/// Where updates come from.
pub trait UpdateSource: Send + Sync {
    /// What installing a found update needs.
    type Handle: Send + Sync;

    /// Asks whether a version newer than the running one exists.
    ///
    /// # Errors
    ///
    /// Returns a [`CheckError`] if the answer couldn't be had in time.
    fn check(
        &self,
    ) -> impl Future<Output = Result<Option<AvailableUpdate<Self::Handle>>, CheckError>> + Send;

    /// Downloads the update, reporting `progress` in percent, checks its
    /// signature and starts the installer. On success the app exits for the
    /// installer and this never returns.
    ///
    /// # Errors
    ///
    /// Returns an [`InstallError`] if the download, the signature check or
    /// starting the installer failed. Nothing was installed then.
    fn install(
        &self,
        handle: &Self::Handle,
        progress: &(dyn Fn(u8) + Send + Sync),
    ) -> impl Future<Output = Result<Infallible, InstallError>> + Send;
}

/// Lets an install wait until no rewrite or replace is running, so the app
/// never quits halfway through one and leaves the clipboard unrestored.
pub trait BusyGate: Send + Sync {
    /// Keeps new rewrites from starting while it lives.
    type Hold: Send;

    /// Waits until nothing is running, closes the popup and holds the gate
    /// until the returned hold is dropped.
    fn hold_when_idle(&self) -> impl Future<Output = Self::Hold> + Send;
}

/// The Windows notifications about updates.
pub trait Notifier: Send + Sync {
    /// Says that `version` can be installed.
    fn update_available(&self, version: &str);

    /// Says that the app now runs `version`, after an update restarted it.
    fn updated_to(&self, version: &str);
}

/// Shows the updates state in Settings and the tray.
pub trait UpdatesView: Send + Sync {
    /// Shows `state`. Called only when the state changes.
    fn render(&self, state: &UpdatesState);
}

impl<T: UpdatesView + ?Sized> UpdatesView for Arc<T> {
    fn render(&self, state: &UpdatesState) {
        (**self).render(state);
    }
}

/// What the controller keeps between runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Remembered {
    /// Whether the app checks for updates by itself.
    pub auto_check: bool,
    /// The newest version an update notification was shown for.
    pub last_notified_version: Option<String>,
    /// The version the app ran last time, if a run was ever recorded.
    pub last_run_version: Option<String>,
}

/// Where the controller keeps what it remembers between runs.
pub trait UpdateMemory: Send + Sync {
    /// What is remembered now.
    fn recall(&self) -> Remembered;

    /// Remembers that an update notification was shown for `version`.
    fn remember_notified(&self, version: &str);

    /// Remembers that the app ran `version`.
    fn remember_run(&self, version: &str);
}

/// The controller's own state, richer than the state it shows.
enum Phase<H> {
    /// Nothing is known yet.
    Idle,
    /// A check is running.
    Checking,
    /// The last check found nothing newer.
    UpToDate,
    /// This update can be installed.
    Available(AvailableUpdate<H>),
    /// The last check failed.
    CheckFailed,
    /// Update now is waiting to install, or downloading.
    Downloading { percent: u8 },
    /// Installing this update failed; it can be tried again.
    InstallFailed(AvailableUpdate<H>),
}

impl<H> Phase<H> {
    /// The state this phase shows.
    fn shown(&self) -> UpdatesState {
        match self {
            Self::Idle => UpdatesState::Idle,
            Self::Checking => UpdatesState::Checking,
            Self::UpToDate => UpdatesState::UpToDate,
            Self::Available(update) => UpdatesState::Available {
                version: update.version.clone(),
            },
            Self::CheckFailed => UpdatesState::Failed {
                step: FailedStep::Check,
            },
            Self::Downloading { percent } => UpdatesState::Downloading { percent: *percent },
            Self::InstallFailed(update) => UpdatesState::InstallFailed {
                version: update.version.clone(),
            },
        }
    }
}

/// Checks for updates and shows what it finds.
pub struct UpdateController<U: UpdateSource, N, V, G, M> {
    /// The version the app runs now, such as `1.1.0`.
    running_version: String,
    source: U,
    notifier: N,
    view: V,
    gate: G,
    memory: M,
    phase: Mutex<Phase<U::Handle>>,
    /// Whether automatic checks are on. The schedule watches it.
    auto_check: watch::Sender<bool>,
}

impl<U, N, V, G, M> UpdateController<U, N, V, G, M>
where
    U: UpdateSource,
    N: Notifier,
    V: UpdatesView,
    G: BusyGate,
    M: UpdateMemory,
{
    /// A controller for an app running `running_version` that knows of no
    /// update yet. Whether automatic checks are on comes from `memory`.
    pub fn new(
        running_version: impl Into<String>,
        source: U,
        notifier: N,
        view: V,
        gate: G,
        memory: M,
    ) -> Self {
        let (auto_check, _) = watch::channel(memory.recall().auto_check);
        Self {
            running_version: running_version.into(),
            source,
            notifier,
            view,
            gate,
            memory,
            phase: Mutex::new(Phase::Idle),
            auto_check,
        }
    }

    /// The state shown now.
    pub fn state(&self) -> UpdatesState {
        self.lock().shown()
    }

    /// Reacts to `event`. A check returns once it has an answer;
    /// [`UpdateEvent::Started`] never returns.
    pub async fn handle(&self, event: UpdateEvent) {
        match event {
            UpdateEvent::Started => {
                self.greet_update();
                self.run_schedule().await;
            }
            UpdateEvent::CheckRequested => self.check().await,
            UpdateEvent::AutoCheckChanged(on) => {
                self.auto_check.send_replace(on);
            }
            UpdateEvent::UpdateNowRequested => self.update_now().await,
        }
    }

    /// Installs the offered update: waits until no rewrite or replace runs,
    /// then downloads with progress and restarts on the new version. If
    /// that fails, offers the update again and lets rewrites run.
    async fn update_now(&self) {
        let update = {
            let mut phase = self.lock();
            if !matches!(*phase, Phase::Available(_) | Phase::InstallFailed(_)) {
                // Nothing to install, or already installing.
                return;
            }
            match std::mem::replace(&mut *phase, Phase::Downloading { percent: 0 }) {
                Phase::Available(update) | Phase::InstallFailed(update) => update,
                _ => unreachable!("matched above"),
            }
        };
        self.view.render(&UpdatesState::Downloading { percent: 0 });
        let hold = self.gate.hold_when_idle().await;
        let progress = |percent: u8| self.progress(percent);
        let Err(error) = self.source.install(&update.handle, &progress).await;
        tracing::warn!(kind = %error, "the update couldn't be installed");
        let failed = Phase::InstallFailed(update);
        let shown = failed.shown();
        *self.lock() = failed;
        self.view.render(&shown);
        drop(hold);
    }

    /// Shows download progress, when it changed.
    fn progress(&self, percent: u8) {
        let percent = percent.min(100);
        {
            let mut phase = self.lock();
            match &mut *phase {
                Phase::Downloading { percent: shown } if *shown != percent => *shown = percent,
                _ => return,
            }
        }
        self.view.render(&UpdatesState::Downloading { percent });
    }

    /// A check the user asked for: shows checking, then what it found.
    /// Never notifies, since the user is already looking at Settings.
    async fn check(&self) {
        {
            let mut phase = self.lock();
            if matches!(*phase, Phase::Checking) {
                // One check at a time; the running one will answer.
                return;
            }
            if matches!(*phase, Phase::Downloading { .. }) {
                // The update being installed is already the newest.
                return;
            }
            *phase = Phase::Checking;
        }
        self.view.render(&UpdatesState::Checking);
        let found = match self.source.check().await {
            Ok(Some(update)) => Phase::Available(update),
            Ok(None) => Phase::UpToDate,
            Err(error) => {
                tracing::warn!(kind = %error, "the update check failed");
                Phase::CheckFailed
            }
        };
        let shown = found.shown();
        *self.lock() = found;
        self.view.render(&shown);
    }

    /// Checks automatically while automatic checks are on: first
    /// [`FIRST_CHECK_DELAY`] after they are on, then every
    /// [`CHECK_INTERVAL`]. Turning them off cancels the wait; turning them
    /// back on starts over. Returns only if the controller is gone.
    async fn run_schedule(&self) {
        let mut on = self.auto_check.subscribe();
        loop {
            if on.wait_for(|on| *on).await.is_err() {
                return;
            }
            let mut delay = FIRST_CHECK_DELAY;
            while wait_while_on(delay, &mut on).await {
                self.check_automatically().await;
                delay = CHECK_INTERVAL;
            }
        }
    }

    /// A check nobody asked for. Shows only what it finds, never
    /// "Checking…", and a failure shows nothing at all. A version newer
    /// than any announced before gets one notification.
    async fn check_automatically(&self) {
        if !self.lock().takes_automatic_result() {
            return;
        }
        let found = match self.source.check().await {
            Ok(found) => found,
            Err(error) => {
                tracing::warn!(kind = %error, "the automatic update check failed");
                return;
            }
        };
        if let Some(update) = &found {
            self.announce(&update.version);
        }
        let shown = {
            let mut phase = self.lock();
            if !phase.takes_automatic_result() {
                return;
            }
            let before = phase.shown();
            *phase = found.map_or(Phase::UpToDate, Phase::Available);
            let after = phase.shown();
            (after != before).then_some(after)
        };
        if let Some(shown) = shown {
            self.view.render(&shown);
        }
    }

    /// Says "Updated to" the running version if the last run was an older
    /// one, then remembers this run. A first run (no run recorded, as after
    /// a fresh install or on a settings file from before this) says nothing.
    fn greet_update(&self) {
        let running = self.running_version.as_str();
        let last = self.memory.recall().last_run_version;
        if last.is_some_and(|last| is_newer(running, &last)) {
            self.notifier.updated_to(running);
        }
        self.memory.remember_run(running);
    }

    /// Notifies about `version` if none was announced yet or it is newer than
    /// the last one announced, then remembers it.
    fn announce(&self, version: &str) {
        let last = self.memory.recall().last_notified_version;
        let new = match last {
            Some(last) => is_newer(version, &last),
            None => semver::Version::parse(version).is_ok(),
        };
        if new {
            self.notifier.update_available(version);
            self.memory.remember_notified(version);
        }
    }

    fn lock(&self) -> MutexGuard<'_, Phase<U::Handle>> {
        self.phase.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl<H> Phase<H> {
    /// Whether an automatic check may replace this phase with what it
    /// found.
    fn takes_automatic_result(&self) -> bool {
        match self {
            Self::Idle | Self::UpToDate | Self::Available(_) | Self::CheckFailed => true,
            // A check the user asked for is answering, or Update now is
            // installing or has just failed (its message stays).
            Self::Checking | Self::Downloading { .. } | Self::InstallFailed(_) => false,
        }
    }
}

/// Whether `version` is newer than `than`. Plain semver, so 1.10.0 is newer
/// than 1.9.0. A version that can't be read is never newer, nor is anything
/// newer than one.
fn is_newer(version: &str, than: &str) -> bool {
    match (
        semver::Version::parse(version),
        semver::Version::parse(than),
    ) {
        (Ok(version), Ok(than)) => version > than,
        _ => false,
    }
}

/// Waits `delay`. Returns `false` early if automatic checks are turned off
/// meanwhile (or the controller is gone), `true` once the delay is over.
async fn wait_while_on(delay: Duration, on: &mut watch::Receiver<bool>) -> bool {
    let mut sleep = pin!(tokio::time::sleep(delay));
    let mut off = pin!(on.wait_for(|on| !*on));
    poll_fn(|cx| {
        if off.as_mut().poll(cx).is_ready() {
            Poll::Ready(false)
        } else if sleep.as_mut().poll(cx).is_ready() {
            Poll::Ready(true)
        } else {
            Poll::Pending
        }
    })
    .await
}

#[cfg(test)]
mod tests;
