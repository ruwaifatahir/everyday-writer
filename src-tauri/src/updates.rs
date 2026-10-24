//! The update controller: the one module that knows the update rules.
//!
//! Like the rewrite session, it reacts to [`UpdateEvent`]s fed to
//! [`UpdateController::handle`] and reaches the outside only through ports,
//! so tests drive it with fakes and no network or Windows:
//!
//! - [`UpdateSource`]: asks GitHub whether a newer version exists;
//! - [`Notifier`]: the Windows notifications about updates;
//! - [`UpdatesView`]: shows each [`UpdatesState`] in Settings and the tray;
//! - [`UpdateMemory`]: what is kept between runs, in the settings file.
//!
//! The controller owns the automatic-check schedule: the first check
//! [`FIRST_CHECK_DELAY`] after the app starts, then every
//! [`CHECK_INTERVAL`], only while automatic checks are on.
//!
//! Privacy: a check sends nothing about the user, and failures are logged
//! only as a fixed error kind, never with URLs or response bodies.

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
}

/// The Windows notifications about updates.
pub trait Notifier: Send + Sync {
    /// Says that `version` can be installed.
    fn update_available(&self, version: &str);
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
}

/// Where the controller keeps what it remembers between runs.
pub trait UpdateMemory: Send + Sync {
    /// What is remembered now.
    fn recall(&self) -> Remembered;

    /// Remembers that an update notification was shown for `version`.
    fn remember_notified(&self, version: &str);
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
        }
    }
}

/// Checks for updates and shows what it finds.
pub struct UpdateController<U: UpdateSource, N, V, M> {
    source: U,
    notifier: N,
    view: V,
    memory: M,
    phase: Mutex<Phase<U::Handle>>,
    /// Whether automatic checks are on. The schedule watches it.
    auto_check: watch::Sender<bool>,
}

impl<U, N, V, M> UpdateController<U, N, V, M>
where
    U: UpdateSource,
    N: Notifier,
    V: UpdatesView,
    M: UpdateMemory,
{
    /// A controller that knows of no update yet. Whether automatic checks
    /// are on comes from `memory`.
    pub fn new(source: U, notifier: N, view: V, memory: M) -> Self {
        let (auto_check, _) = watch::channel(memory.recall().auto_check);
        Self {
            source,
            notifier,
            view,
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
            UpdateEvent::Started => self.run_schedule().await,
            UpdateEvent::CheckRequested => self.check().await,
            UpdateEvent::AutoCheckChanged(on) => {
                self.auto_check.send_replace(on);
            }
        }
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

    /// Notifies about `version` unless it is no newer than the last version
    /// announced, then remembers it.
    fn announce(&self, version: &str) {
        let last = self.memory.recall().last_notified_version;
        if is_newer(version, last.as_deref()) {
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
    /// found: not while a check the user asked for is answering.
    fn takes_automatic_result(&self) -> bool {
        match self {
            Self::Idle | Self::UpToDate | Self::Available(_) | Self::CheckFailed => true,
            Self::Checking => false,
        }
    }
}

/// Whether `found` is newer than the `last` version announced. Compared as
/// versions, so 1.10.0 is newer than 1.9.0. A version that can't be read
/// counts as new unless it is the same text.
fn is_newer(found: &str, last: Option<&str>) -> bool {
    let Some(last) = last else {
        return true;
    };
    match (semver::Version::parse(found), semver::Version::parse(last)) {
        (Ok(found), Ok(last)) => found > last,
        _ => found != last,
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
