//! The update controller as the running app wires it: the Tauri updater as
//! its [`UpdateSource`], Windows notifications as its [`Notifier`], Settings
//! and the tray as its [`UpdatesView`], the rewrite session as its
//! [`BusyGate`], and the commands the Settings page calls.

#![expect(
    clippy::needless_pass_by_value,
    reason = "Tauri commands receive `State` by value"
)]

use std::convert::Infallible;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_notification::{ActionPerformed, NotificationExt};
use tauri_plugin_updater::{Update, UpdaterExt};

use crate::popup::AppSession;
use crate::settings::AppSettings;
use crate::settings_window::{self, CommandError, Section};
use crate::tray;
use crate::updates::{
    AvailableUpdate, BusyGate, CheckError, InstallError, Notifier, Remembered, UpdateController,
    UpdateEvent, UpdateMemory, UpdateSource, UpdatesState, UpdatesView,
};

/// The event that carries each new [`UpdatesState`] to the settings window.
const STATE_EVENT: &str = "updates-state";

/// How long a check may take before it counts as failed.
const CHECK_TIMEOUT: Duration = Duration::from_secs(30);

/// The update controller as the running app wires it.
pub type AppUpdates = UpdateController<
    TauriUpdateSource,
    TauriNotifier,
    Arc<TauriUpdatesView>,
    SessionGate,
    SettingsMemory,
>;

/// Builds the update controller, hands it and its view to Tauri's state,
/// and starts its automatic-check schedule. Call from `setup`, after the
/// settings are loaded and the rewrite session is managed, and before the
/// tray is built or any notification is shown.
pub fn start(app: &AppHandle) {
    open_updates_on_click(app);
    let view = Arc::new(TauriUpdatesView::new(app.clone()));
    app.manage(Arc::clone(&view));
    let session = Arc::clone(app.state::<Arc<AppSession>>().inner());
    let settings = Arc::clone(app.state::<Arc<AppSettings>>().inner());
    let controller: AppUpdates = UpdateController::new(
        app.package_info().version.to_string(),
        TauriUpdateSource(app.clone()),
        TauriNotifier(app.clone()),
        view,
        SessionGate(session),
        SettingsMemory(settings),
    );
    app.manage(Arc::new(controller));
    send(app, UpdateEvent::Started);
}

/// Makes a click on the update-available toast open Settings at Updates.
/// A click is reported only while the toast is on screen, and only for
/// toasts shown after this, so it runs before anything can show one.
///
/// The handler runs on the plugin's own thread, so it only decides and
/// hands the opening to the async runtime. It never installs anything.
fn open_updates_on_click(app: &AppHandle) {
    let handle = app.clone();
    let registered = app.notification().on_action(move |performed| {
        if click_opens_updates(performed) {
            settings_window::open_at_later(&handle, Some(Section::Updates));
        }
    });
    if registered.is_err() {
        tracing::warn!("update notification clicks can't be received");
    }
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

    /// Downloads with no time limit (the check's timeout doesn't apply), then
    /// the updater verifies the signature against the built-in public key,
    /// starts the installer in passive mode and exits the app.
    async fn install(
        &self,
        update: &Update,
        progress: &(dyn Fn(u8) + Send + Sync),
    ) -> Result<Infallible, InstallError> {
        let mut downloaded: u64 = 0;
        update
            .download_and_install(
                |chunk, total| {
                    downloaded = downloaded.saturating_add(chunk as u64);
                    if let Some(total) = total.filter(|total| *total > 0) {
                        let percent = (downloaded.saturating_mul(100) / total).min(100);
                        progress(u8::try_from(percent).unwrap_or(100));
                    }
                },
                || {},
            )
            .await
            .map_err(|error| install_error(&error))?;
        // On Windows a successful install exits the process before this.
        Err(InstallError::Other)
    }
}

/// The kind of a failed install, the only thing logged about it.
fn install_error(error: &tauri_plugin_updater::Error) -> InstallError {
    use tauri_plugin_updater::Error;
    match error {
        Error::Reqwest(_) | Error::Network(_) => InstallError::Network,
        Error::Minisign(_)
        | Error::Base64(_)
        | Error::SignatureUtf8(_)
        | Error::SignedVersionMismatch { .. }
        | Error::MissingSignedVersion => InstallError::Signature,
        _ => InstallError::Other,
    }
}

/// The rewrite session as the busy gate: an install waits for it to be
/// idle and holds it, so the shortcut starts nothing meanwhile.
pub struct SessionGate(Arc<AppSession>);

/// Releases the rewrite session when dropped, after a failed install.
pub struct SessionHold(Arc<AppSession>);

impl Drop for SessionHold {
    fn drop(&mut self) {
        self.0.release();
    }
}

impl BusyGate for SessionGate {
    type Hold = SessionHold;

    async fn hold_when_idle(&self) -> SessionHold {
        self.0.hold_when_idle().await;
        SessionHold(Arc::clone(&self.0))
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

/// The key, in a toast's extra data, that says which toast it is.
const TOAST_KIND: &str = "kind";

/// The [`TOAST_KIND`] of the "update is available" toast.
const UPDATE_AVAILABLE: &str = "update-available";

/// The action id the notification plugin reports for a click on a toast's body.
const TAP: &str = "tap";

/// Whether a reported toast click should open Settings at Updates: only a
/// click on the update-available toast. Once any click handler is
/// registered every toast reports its clicks, so all others are ignored.
fn click_opens_updates(performed: &ActionPerformed) -> bool {
    performed.action_id() == TAP
        && performed
            .notification()
            .and_then(|toast| toast.extra().get(TOAST_KIND))
            .and_then(serde_json::Value::as_str)
            == Some(UPDATE_AVAILABLE)
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
            .extra(TOAST_KIND, UPDATE_AVAILABLE)
            .show();
        if shown.is_err() {
            tracing::warn!("the update notification couldn't be shown");
        }
    }

    fn updated_to(&self, version: &str) {
        let shown = self
            .0
            .notification()
            .builder()
            .title("everyday-writer")
            .body(format!("Updated to {version}"))
            .show();
        if shown.is_err() {
            tracing::warn!("the updated notification couldn't be shown");
        }
    }
}

/// Keeps the update memory in the settings file.
pub struct SettingsMemory(Arc<AppSettings>);

impl UpdateMemory for SettingsMemory {
    fn recall(&self) -> Remembered {
        self.0.settings().updates
    }

    fn remember(&self, change: impl FnOnce(&mut Remembered)) {
        // Never over a damaged file: see `AppSettings::remember`.
        self.0.remember(|settings| change(&mut settings.updates));
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
        .modify(|s| s.updates.auto_check = enabled)
        .map_err(|_| CommandError::SaveSettings)?;
    // Handled in place, not spawned, so quick toggles stay in order.
    controller
        .handle(UpdateEvent::AutoCheckChanged(enabled))
        .await;
    Ok(())
}

/// The Settings page's Update now button: installs the offered update and
/// restarts. Progress and failure arrive as `updates-state` events.
#[tauri::command]
pub fn update_now(app: AppHandle) {
    send(&app, UpdateEvent::UpdateNowRequested);
}

#[cfg(test)]
mod tests {
    use tauri_plugin_notification::ActionPerformed;

    use super::click_opens_updates;

    fn click(json: serde_json::Value) -> ActionPerformed {
        serde_json::from_value(json).unwrap()
    }

    #[test]
    fn a_click_on_the_update_toast_opens_updates() {
        let performed = click(serde_json::json!({
            "actionId": "tap",
            "notification": { "id": 7, "extra": { "kind": "update-available" } }
        }));
        assert!(click_opens_updates(&performed));
    }

    #[test]
    fn a_click_on_the_shortcut_failed_toast_does_nothing() {
        let performed = click(serde_json::json!({
            "actionId": "tap",
            "notification": {
                "id": 3,
                "title": "everyday-writer",
                "body": "Your shortcut is used by another app. Pick a new one in Settings."
            }
        }));
        assert!(!click_opens_updates(&performed));
    }

    #[test]
    fn a_click_whose_toast_cant_be_read_does_nothing() {
        let performed = click(serde_json::json!({ "actionId": "tap" }));
        assert!(!click_opens_updates(&performed));
    }

    #[test]
    fn an_action_button_on_the_update_toast_does_nothing() {
        let performed = click(serde_json::json!({
            "actionId": "install",
            "notification": { "id": 7, "extra": { "kind": "update-available" } }
        }));
        assert!(!click_opens_updates(&performed));
    }
}
