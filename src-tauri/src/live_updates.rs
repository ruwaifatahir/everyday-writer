//! The update controller as the running app wires it: the Tauri updater as
//! its [`UpdateSource`], Windows notifications as its [`Notifier`], Settings
//! and the tray as its [`UpdatesView`], and the commands the Settings page
//! calls.

#![expect(
    clippy::needless_pass_by_value,
    reason = "Tauri commands receive `State` by value"
)]

use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_updater::{Update, UpdaterExt};

use crate::settings::AppSettings;
use crate::settings_window::{self, CommandError};
use crate::tray;
use crate::updates::{
    AvailableUpdate, CheckError, Notifier, Remembered, UpdateController, UpdateEvent, UpdateMemory,
    UpdateSource, UpdatesState, UpdatesView,
};

/// The event that carries each new [`UpdatesState`] to the settings window.
const STATE_EVENT: &str = "updates-state";

/// How long a check may take before it counts as failed.
const CHECK_TIMEOUT: Duration = Duration::from_secs(30);

/// The update controller as the running app wires it.
pub type AppUpdates =
    UpdateController<TauriUpdateSource, TauriNotifier, Arc<TauriUpdatesView>, SettingsMemory>;

/// Builds the update controller, hands it and its view to Tauri's state,
/// and starts its automatic-check schedule. Call from `setup`, after the
/// settings are loaded and before the tray is built.
pub fn start(app: &AppHandle) {
    let view = Arc::new(TauriUpdatesView::new(app.clone()));
    app.manage(Arc::clone(&view));
    let settings = Arc::clone(app.state::<Arc<AppSettings>>().inner());
    let controller: AppUpdates = UpdateController::new(
        TauriUpdateSource(app.clone()),
        TauriNotifier(app.clone()),
        view,
        SettingsMemory(settings),
    );
    app.manage(Arc::new(controller));
    send(app, UpdateEvent::Started);
}

/// Hands `event` to the update controller on the async runtime, so menu
/// and command callbacks return at once.
pub fn send(app: &AppHandle, event: UpdateEvent) {
    let controller = Arc::clone(app.state::<Arc<AppUpdates>>().inner());
    tauri::async_runtime::spawn(async move { controller.handle(event).await });
}

/// Asks the Tauri updater, which fetches the static `latest.json` from
/// tauri.conf.json: a plain GET with no app or machine data, verified
/// against the built-in public key on install.
pub struct TauriUpdateSource(AppHandle);

impl UpdateSource for TauriUpdateSource {
    type Handle = Update;

    async fn check(&self) -> Result<Option<AvailableUpdate<Update>>, CheckError> {
        let updater = self.0.updater().map_err(|_| CheckError::Other)?;
        let found = tokio::time::timeout(CHECK_TIMEOUT, updater.check())
            .await
            .map_err(|_| CheckError::Timeout)?
            .map_err(|error| check_error(&error))?;
        Ok(found.map(|update| AvailableUpdate {
            version: update.version.clone(),
            handle: update,
        }))
    }
}

/// The kind of a failed check. The updater's own message can hold URLs,
/// so only this kind is ever logged.
fn check_error(error: &tauri_plugin_updater::Error) -> CheckError {
    use tauri_plugin_updater::Error;
    match error {
        Error::Reqwest(_) | Error::Network(_) => CheckError::Network,
        Error::Serialization(_)
        | Error::ReleaseNotFound
        | Error::TargetNotFound(_)
        | Error::TargetsNotFound(_)
        | Error::Semver(_)
        | Error::UrlParse(_) => CheckError::BadResponse,
        _ => CheckError::Other,
    }
}

/// Shows update notifications as Windows toasts.
pub struct TauriNotifier(AppHandle);

impl Notifier for TauriNotifier {
    fn update_available(&self, version: &str) {
        let shown = self
            .0
            .notification()
            .builder()
            .title("everyday-writer")
            .body(format!("everyday-writer {version} is available"))
            .show();
        if shown.is_err() {
            tracing::warn!("the update notification couldn't be shown");
        }
    }
}

/// Keeps the update memory in the settings file.
pub struct SettingsMemory(Arc<AppSettings>);

impl UpdateMemory for SettingsMemory {
    fn recall(&self) -> Remembered {
        let settings = self.0.settings();
        Remembered {
            auto_check: settings.auto_check,
            last_notified_version: settings.last_notified_version,
        }
    }

    fn remember_notified(&self, version: &str) {
        // Never over a damaged file: see `AppSettings::remember`.
        self.0
            .remember(|settings| settings.last_notified_version = Some(version.to_owned()));
    }
}

/// Shows the updates state in the settings window and the tray item.
pub struct TauriUpdatesView {
    app: AppHandle,
    state: Mutex<UpdatesState>,
}

impl TauriUpdatesView {
    /// A view that shows nothing known yet.
    pub fn new(app: AppHandle) -> Self {
        Self {
            app,
            state: Mutex::new(UpdatesState::Idle),
        }
    }

    /// The state shown now.
    pub fn state(&self) -> UpdatesState {
        self.state
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

impl UpdatesView for TauriUpdatesView {
    fn render(&self, state: &UpdatesState) {
        *self.state.lock().unwrap_or_else(PoisonError::into_inner) = state.clone();
        tray::show_updates(&self.app, state);
        if self
            .app
            .get_webview_window(settings_window::LABEL)
            .is_some()
            && self
                .app
                .emit_to(settings_window::LABEL, STATE_EVENT, state)
                .is_err()
        {
            tracing::warn!("the updates state couldn't be sent");
        }
    }
}

/// The updates state, for a Settings page that loads after it was sent.
#[tauri::command(async)]
pub fn get_updates_state(view: State<'_, Arc<TauriUpdatesView>>) -> UpdatesState {
    view.state()
}

/// The Settings page's Check for updates button. The answer arrives as
/// `updates-state` events.
#[tauri::command]
pub fn check_for_updates(app: AppHandle) {
    send(&app, UpdateEvent::CheckRequested);
}

/// The Settings page's "Check for updates automatically" switch. Saves the
/// choice, then starts or stops the schedule at once. A known update stays
/// offered either way.
///
/// # Errors
///
/// Returns [`CommandError::SaveSettings`] if the choice can't be saved; the
/// schedule stays as it was then.
#[tauri::command]
pub async fn set_auto_check(
    settings: State<'_, Arc<AppSettings>>,
    controller: State<'_, Arc<AppUpdates>>,
    enabled: bool,
) -> Result<(), CommandError> {
    settings
        .modify(|s| s.auto_check = enabled)
        .map_err(|_| CommandError::SaveSettings)?;
    // Handled in place, not spawned, so quick toggles stay in order.
    controller
        .handle(UpdateEvent::AutoCheckChanged(enabled))
        .await;
    Ok(())
}
