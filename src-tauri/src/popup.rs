//! The popup window: the rewrite session's [`PopupView`] port, the commands
//! the popup calls, and the session as the running app wires it.
//!
//! The popup has no logic of its own. It shows the state the session sends
//! as the `popup-state` event and turns its buttons into session events.

#![expect(
    clippy::needless_pass_by_value,
    reason = "Tauri commands receive `State` by value"
)]

mod placement;

use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use tauri::window::Color;
use tauri::{
    AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, State, Theme, WebviewUrl,
    WebviewWindow, WebviewWindowBuilder, WindowEvent,
};

use crate::live_rewriter::LiveRewriter;
use crate::session::{Event, PopupState, PopupView, RewriteSession};
use crate::settings::{AppSettings, KeyringApiKeys};
use crate::settings_window;
use crate::text_bridge::WindowsTextBridge;
use placement::{GAP, Rect, place_near_cursor};

/// The popup page background (`--cream` in `src/styles/tokens.css`),
/// painted natively so the window never flashes white before the page loads.
const BACKGROUND: Color = Color(0xff, 0xff, 0xeb, 0xff);

/// Label of the popup window. The `popup` capability is granted to it.
pub const LABEL: &str = "popup";

/// The event that carries each new [`PopupState`] to the popup window.
const STATE_EVENT: &str = "popup-state";

/// The popup's size in logical pixels.
const WIDTH: f64 = 400.0;
const HEIGHT: f64 = 220.0;

/// Focus changes right after the popup opens come from Windows settling
/// focus, not from the user clicking elsewhere.
const BLUR_GRACE: Duration = Duration::from_millis(250);

/// The rewrite session as the running app wires it.
pub type AppSession = RewriteSession<
    WindowsTextBridge,
    LiveRewriter<Arc<AppSettings>, KeyringApiKeys>,
    Arc<AppSettings>,
    Arc<TauriPopup>,
>;

/// Creates the hidden popup window, so showing it later is instant. Call
/// once, from `setup`.
///
/// # Errors
///
/// Returns an error if the window can't be created.
pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let window = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("popup.html".into()))
        .title(&app.package_info().name)
        .inner_size(WIDTH, HEIGHT)
        .decorations(false)
        .theme(Some(Theme::Light))
        .background_color(BACKGROUND)
        .resizable(false)
        .maximizable(false)
        .minimizable(false)
        .shadow(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(false)
        .focused(false)
        .build()?;
    let app = app.clone();
    window.on_window_event(move |event| match event {
        WindowEvent::Focused(false) => {
            let popup = app.state::<Arc<TauriPopup>>();
            if !popup.just_opened() {
                send(&app, Event::Close);
            }
        }
        WindowEvent::CloseRequested { api, .. } => {
            // Alt+F4: the popup is only ever hidden, never destroyed.
            api.prevent_close();
            send(&app, Event::Close);
        }
        _ => {}
    });
    Ok(())
}

/// Hands `event` to the rewrite session on the async runtime, so window
/// and shortcut callbacks return at once.
pub fn send(app: &AppHandle, event: Event) {
    let session = Arc::clone(app.state::<Arc<AppSession>>().inner());
    tauri::async_runtime::spawn(async move { session.handle(event).await });
}

/// Shows session states in the popup window.
pub struct TauriPopup {
    app: AppHandle,
    state: Mutex<PopupState>,
    opened_at: Mutex<Option<Instant>>,
}

impl TauriPopup {
    /// A view of the popup window in `app`.
    pub fn new(app: AppHandle) -> Self {
        Self {
            app,
            state: Mutex::new(PopupState::Hidden),
            opened_at: Mutex::new(None),
        }
    }

    /// The state shown now.
    pub fn state(&self) -> PopupState {
        self.state
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    fn just_opened(&self) -> bool {
        self.opened_at
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .is_some_and(|opened| opened.elapsed() < BLUR_GRACE)
    }

    /// Moves the popup next to the mouse, shows it and gives it focus.
    fn open(&self, window: &WebviewWindow) -> tauri::Result<()> {
        if let Some((position, size)) = self.placement()? {
            window.set_position(position)?;
            // Moving onto a monitor with another scale makes Windows resize
            // the window, so set the size and position once more.
            window.set_size(size)?;
            window.set_position(position)?;
        }
        *self
            .opened_at
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = Some(Instant::now());
        window.show()?;
        window.set_focus()
    }

    /// Where the popup goes, in physical pixels of the monitor under the
    /// mouse, fully inside that monitor's work area.
    fn placement(&self) -> tauri::Result<Option<(PhysicalPosition<i32>, PhysicalSize<u32>)>> {
        let cursor = self.app.cursor_position()?;
        let monitor = match self.app.monitor_from_point(cursor.x, cursor.y)? {
            Some(monitor) => Some(monitor),
            None => self.app.primary_monitor()?,
        };
        let Some(monitor) = monitor else {
            return Ok(None);
        };
        let scale = monitor.scale_factor();
        let work_area = monitor.work_area();
        let area = Rect {
            x: work_area.position.x,
            y: work_area.position.y,
            width: saturating_i32(work_area.size.width),
            height: saturating_i32(work_area.size.height),
        };
        let size = (to_pixels(WIDTH * scale), to_pixels(HEIGHT * scale));
        let (x, y) = place_near_cursor(
            (to_pixels(cursor.x), to_pixels(cursor.y)),
            size,
            to_pixels(f64::from(GAP) * scale),
            area,
        );
        let physical_size = PhysicalSize::new(
            u32::try_from(size.0).unwrap_or_default(),
            u32::try_from(size.1).unwrap_or_default(),
        );
        Ok(Some((PhysicalPosition::new(x, y), physical_size)))
    }
}

impl PopupView for TauriPopup {
    fn render(&self, state: &PopupState) {
        *self.state.lock().unwrap_or_else(PoisonError::into_inner) = state.clone();
        let Some(window) = self.app.get_webview_window(LABEL) else {
            tracing::warn!("the popup window is missing");
            return;
        };
        // The page gets the new state before the window appears.
        if self.app.emit_to(LABEL, STATE_EVENT, state).is_err() {
            tracing::warn!("the popup state couldn't be sent");
        }
        let shown = if *state == PopupState::Hidden {
            window.hide()
        } else if window.is_visible().unwrap_or(false) {
            Ok(())
        } else {
            self.open(&window)
        };
        if shown.is_err() {
            tracing::warn!("the popup couldn't be shown or hidden");
        }
    }

    fn open_settings(&self) {
        if settings_window::open_at(&self.app, None).is_err() {
            tracing::warn!("the settings window could not open");
        }
    }
}

/// Rounds a physical pixel value. Screen coordinates always fit in `i32`.
#[expect(
    clippy::cast_possible_truncation,
    reason = "screen coordinates fit in i32, and `as` saturates"
)]
fn to_pixels(value: f64) -> i32 {
    value.round() as i32
}

fn saturating_i32(value: u32) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

/// The popup's current state, for a page that loads after it was sent.
#[tauri::command(async)]
pub fn popup_state(popup: State<'_, Arc<TauriPopup>>) -> PopupState {
    popup.state()
}

/// The Replace button or Enter.
#[tauri::command]
pub fn popup_replace(app: AppHandle) {
    send(&app, Event::Replace);
}

/// The Copy button.
#[tauri::command]
pub fn popup_copy(app: AppHandle) {
    send(&app, Event::Copy);
}

/// The ✕ button or Esc.
#[tauri::command]
pub fn popup_close(app: AppHandle) {
    send(&app, Event::Close);
}

/// A message's Open Settings button.
#[tauri::command]
pub fn popup_open_settings(app: AppHandle) {
    send(&app, Event::OpenSettings);
}
