//! The update controller: the one module that knows the update rules.
//!
//! Like the rewrite session, it reacts to [`UpdateEvent`]s fed to
//! [`UpdateController::handle`] and reaches the outside only through ports,
//! so tests drive it with fakes and no network or Windows:
//!
//! - [`UpdateSource`]: asks GitHub whether a newer version exists;
//! - [`Notifier`]: the Windows notifications about updates;
//! - [`UpdatesView`]: shows each [`UpdatesState`] in Settings and the tray.
//!
//! Privacy: a check sends nothing about the user, and failures are logged
//! only as a fixed error kind, never with URLs or response bodies.

use std::future::Future;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use serde::Serialize;

/// Something that asks the controller to act.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateEvent {
    /// The user asked for a check: the tray item or the Settings button.
    CheckRequested,
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
pub struct UpdateController<U: UpdateSource, N, V> {
    source: U,
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "only automatic checks notify, and manual ones come first"
        )
    )]
    notifier: N,
    view: V,
    phase: Mutex<Phase<U::Handle>>,
}

impl<U, N, V> UpdateController<U, N, V>
where
    U: UpdateSource,
    N: Notifier,
    V: UpdatesView,
{
    /// A controller that knows of no update yet.
    pub fn new(source: U, notifier: N, view: V) -> Self {
        Self {
            source,
            notifier,
            view,
            phase: Mutex::new(Phase::Idle),
        }
    }

    /// The state shown now.
    pub fn state(&self) -> UpdatesState {
        self.lock().shown()
    }

    /// Reacts to `event`. A check returns once it has an answer.
    pub async fn handle(&self, event: UpdateEvent) {
        match event {
            UpdateEvent::CheckRequested => self.check().await,
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

    fn lock(&self) -> MutexGuard<'_, Phase<U::Handle>> {
        self.phase.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[cfg(test)]
mod tests;
