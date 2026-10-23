//! Start with Windows: keeps the Windows startup entry in line with the
//! setting, which is on by default.
//!
//! The settings file is the source of truth. The startup entry is written
//! again at every launch, so it follows the app if the exe moves.

use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

/// The startup entry couldn't be changed. Holds no detail.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("the Windows startup entry couldn't be changed")]
pub struct AutostartError;

/// Turning start-with-Windows on or off failed. The setting is unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ChangeError {
    /// Windows refused the change.
    #[error(transparent)]
    Autostart(#[from] AutostartError),
    /// The setting couldn't be saved, so Windows was put back.
    #[error("the setting couldn't be saved")]
    Save,
}

/// The Windows startup entry. A port, so the rules can be tested without
/// the registry.
pub trait Autostart {
    /// Adds the startup entry if `enabled`, or removes it.
    ///
    /// # Errors
    ///
    /// Returns [`AutostartError`] if Windows refuses.
    fn set(&self, enabled: bool) -> Result<(), AutostartError>;
}

/// Turns start-with-Windows on or off: Windows first, then `save`. If saving
/// fails, the startup entry is put back so it matches the saved setting.
///
/// # Errors
///
/// Returns [`ChangeError::Autostart`] if Windows refuses and
/// [`ChangeError::Save`] if `save` fails.
pub fn change<E>(
    autostart: &impl Autostart,
    enabled: bool,
    save: impl FnOnce(bool) -> Result<(), E>,
) -> Result<(), ChangeError> {
    autostart.set(enabled)?;
    if save(enabled).is_err() {
        if autostart.set(!enabled).is_err() {
            tracing::warn!("the Windows startup entry couldn't be put back");
        }
        return Err(ChangeError::Save);
    }
    Ok(())
}

/// The real startup entry, through the autostart plugin (a value under
/// `HKCU\...\CurrentVersion\Run`).
///
/// Debug builds never touch it, so a development build in `target/debug`
/// never starts with Windows.
pub struct TauriAutostart(pub AppHandle);

impl Autostart for TauriAutostart {
    fn set(&self, enabled: bool) -> Result<(), AutostartError> {
        if cfg!(debug_assertions) {
            return Ok(());
        }
        let launcher = self.0.autolaunch();
        let result = if enabled {
            launcher.enable()
        } else {
            launcher.disable()
        };
        result.map_err(|_| AutostartError)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};

    use super::{Autostart, AutostartError, ChangeError, change};

    /// A startup entry in memory. `broken` refuses every change.
    #[derive(Default)]
    struct FakeAutostart {
        enabled: Cell<bool>,
        broken: bool,
    }

    impl Autostart for FakeAutostart {
        fn set(&self, enabled: bool) -> Result<(), AutostartError> {
            if self.broken {
                return Err(AutostartError);
            }
            self.enabled.set(enabled);
            Ok(())
        }
    }

    fn enabled() -> FakeAutostart {
        FakeAutostart {
            enabled: Cell::new(true),
            ..FakeAutostart::default()
        }
    }

    #[test]
    fn turning_it_off_removes_the_startup_entry_and_saves() {
        let autostart = enabled();
        let saved = RefCell::new(None);

        change(&autostart, false, |on| {
            *saved.borrow_mut() = Some(on);
            Ok::<(), ()>(())
        })
        .unwrap();

        assert!(!autostart.enabled.get());
        assert_eq!(saved.into_inner(), Some(false));
    }

    #[test]
    fn turning_it_on_adds_the_startup_entry() {
        let autostart = FakeAutostart::default();

        change(&autostart, true, |_| Ok::<(), ()>(())).unwrap();

        assert!(autostart.enabled.get());
    }

    #[test]
    fn a_refused_change_is_not_saved() {
        let autostart = FakeAutostart {
            broken: true,
            ..FakeAutostart::default()
        };
        let saved = RefCell::new(None);

        let result = change(&autostart, false, |on| {
            *saved.borrow_mut() = Some(on);
            Ok::<(), ()>(())
        });

        assert_eq!(result, Err(ChangeError::Autostart(AutostartError)));
        assert_eq!(saved.into_inner(), None);
    }

    #[test]
    fn a_change_that_cannot_be_saved_puts_the_startup_entry_back() {
        let autostart = enabled();

        let result = change(&autostart, false, |_| Err(()));

        assert_eq!(result, Err(ChangeError::Save));
        assert!(autostart.enabled.get());
    }
}
