//! Global shortcut: registers the rewrite shortcut, changes it, and reports
//! each press.
//!
//! The `global-shortcut` plugin is registered in [`crate::run`].

use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

/// The rewrite shortcut until the user picks another one.
pub const DEFAULT_SHORTCUT: &str = "Ctrl+Shift+Space";

/// The shortcut couldn't be registered. Holds no detail.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ShortcutError {
    /// The text isn't a valid shortcut.
    #[error("the shortcut is not valid")]
    Invalid,
    /// Windows refused it, almost always because another app uses it.
    #[error("the shortcut is used by another app")]
    Taken,
}

/// The shortcut couldn't be changed. The previous one still works.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ChangeError {
    /// The new shortcut was refused.
    #[error(transparent)]
    Shortcut(#[from] ShortcutError),
    /// The new shortcut worked but couldn't be saved, so it was dropped.
    #[error("the shortcut couldn't be saved")]
    Save,
}

/// Parses `text` into a shortcut the app accepts: Ctrl, Alt or Win, any
/// Shift, and one key. Shift alone isn't enough, as it would take over a
/// capital letter or symbol while the user types.
///
/// # Errors
///
/// Returns [`ShortcutError::Invalid`] if `text` isn't such a shortcut.
pub fn parse(text: &str) -> Result<Shortcut, ShortcutError> {
    let shortcut: Shortcut = text.trim().parse().map_err(|_| ShortcutError::Invalid)?;
    let required = tauri_plugin_global_shortcut::Modifiers::CONTROL
        | tauri_plugin_global_shortcut::Modifiers::ALT
        | tauri_plugin_global_shortcut::Modifiers::SUPER;
    if shortcut.mods.intersects(required) {
        Ok(shortcut)
    } else {
        Err(ShortcutError::Invalid)
    }
}

/// Whether `text` is [`DEFAULT_SHORTCUT`], however it's written. Compares
/// parsed shortcuts, since the same keys can be spelled many ways.
pub fn is_default(text: &str) -> bool {
    parse(text).is_ok_and(|shortcut| parse(DEFAULT_SHORTCUT) == Ok(shortcut))
}

/// The system's global shortcuts. A port, so the change rules can be tested
/// without Windows.
pub trait Hotkeys {
    /// Registers `shortcut` so pressing it starts a rewrite.
    ///
    /// # Errors
    ///
    /// Returns [`ShortcutError::Taken`] if Windows refuses it.
    fn register(&self, shortcut: Shortcut) -> Result<(), ShortcutError>;

    /// Stops listening for `shortcut`. Does nothing if it isn't registered.
    fn unregister(&self, shortcut: Shortcut);

    /// Whether this app has `shortcut` registered.
    fn is_registered(&self, shortcut: Shortcut) -> bool;
}

/// Changes the rewrite shortcut from `current` to `new`, never leaving the
/// user without a working one: `new` is registered first, then saved with
/// `save`, and only then is `current` released. Picking the shortcut that's
/// already in use is a no-op, unless it isn't registered (it failed at
/// startup or is paused), in which case it is tried again. A paused
/// `current` has nothing to release.
///
/// # Errors
///
/// Returns [`ChangeError::Shortcut`] if `new` is invalid or taken, and
/// [`ChangeError::Save`] if saving fails. `current` stays registered then.
pub fn change<E>(
    hotkeys: &impl Hotkeys,
    current: &str,
    new: &str,
    save: impl FnOnce(&str) -> Result<(), E>,
) -> Result<(), ChangeError> {
    let new_shortcut = parse(new)?;
    let current_shortcut = parse(current).ok();
    let unchanged = current_shortcut == Some(new_shortcut);
    if unchanged && hotkeys.is_registered(new_shortcut) {
        return Ok(());
    }
    hotkeys.register(new_shortcut)?;
    if save(new.trim()).is_err() {
        if !unchanged {
            hotkeys.unregister(new_shortcut);
        }
        return Err(ChangeError::Save);
    }
    if let Some(current_shortcut) = current_shortcut.filter(|_| !unchanged) {
        hotkeys.unregister(current_shortcut);
    }
    Ok(())
}

/// Pauses the `saved` shortcut while Settings records a new one, so pressing
/// it reaches the field instead of starting a rewrite. [`resume`] brings it
/// back. Does nothing if `saved` isn't a valid shortcut.
pub fn pause(hotkeys: &impl Hotkeys, saved: &str) {
    if let Ok(saved) = parse(saved) {
        hotkeys.unregister(saved);
    }
}

/// Brings back the `saved` shortcut after [`pause`]. Does nothing if it
/// already works, so it is safe to call whenever recording might have ended.
///
/// # Errors
///
/// Returns [`ShortcutError::Invalid`] if `saved` isn't a valid shortcut and
/// [`ShortcutError::Taken`] if another app took it while it was paused.
pub fn resume(hotkeys: &impl Hotkeys, saved: &str) -> Result<(), ShortcutError> {
    let saved = parse(saved)?;
    if hotkeys.is_registered(saved) {
        return Ok(());
    }
    hotkeys.register(saved)
}

/// The real global shortcuts, through the Tauri plugin. Each press calls
/// `on_press`, which runs on the main thread and must return quickly.
pub struct TauriHotkeys {
    app: AppHandle,
    on_press: fn(&AppHandle),
}

impl TauriHotkeys {
    /// Global shortcuts that call `on_press` when pressed.
    pub fn new(app: AppHandle, on_press: fn(&AppHandle)) -> Self {
        Self { app, on_press }
    }
}

impl Hotkeys for TauriHotkeys {
    /// Any error from a valid shortcut counts as taken: `RegisterHotKey`
    /// failing on a valid combination almost always means another app holds
    /// it. The plugin reports errors only as text, so they aren't inspected.
    fn register(&self, shortcut: Shortcut) -> Result<(), ShortcutError> {
        let on_press = self.on_press;
        self.app
            .global_shortcut()
            .on_shortcut(shortcut, move |app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    on_press(app);
                }
            })
            .map_err(|_| ShortcutError::Taken)
    }

    fn unregister(&self, shortcut: Shortcut) {
        if self.app.global_shortcut().unregister(shortcut).is_err() {
            tracing::warn!("the old shortcut couldn't be released");
        }
    }

    fn is_registered(&self, shortcut: Shortcut) -> bool {
        self.app.global_shortcut().is_registered(shortcut)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::HashSet;

    use tauri_plugin_global_shortcut::Shortcut;

    use super::{ChangeError, Hotkeys, ShortcutError, change, is_default, parse, pause, resume};

    /// Global shortcuts in memory. `taken` are held by "another app".
    #[derive(Default)]
    struct FakeHotkeys {
        taken: HashSet<Shortcut>,
        registered: RefCell<HashSet<Shortcut>>,
    }

    impl FakeHotkeys {
        fn with(registered: &str) -> Self {
            let hotkeys = Self::default();
            hotkeys.registered.borrow_mut().insert(shortcut(registered));
            hotkeys
        }

        fn taken(mut self, taken: &str) -> Self {
            self.taken.insert(shortcut(taken));
            self
        }

        fn works(&self, text: &str) -> bool {
            self.registered.borrow().contains(&shortcut(text))
        }

        fn count(&self) -> usize {
            self.registered.borrow().len()
        }
    }

    impl Hotkeys for FakeHotkeys {
        fn register(&self, shortcut: Shortcut) -> Result<(), ShortcutError> {
            if self.taken.contains(&shortcut) || !self.registered.borrow_mut().insert(shortcut) {
                return Err(ShortcutError::Taken);
            }
            Ok(())
        }

        fn unregister(&self, shortcut: Shortcut) {
            self.registered.borrow_mut().remove(&shortcut);
        }

        fn is_registered(&self, shortcut: Shortcut) -> bool {
            self.registered.borrow().contains(&shortcut)
        }
    }

    fn shortcut(text: &str) -> Shortcut {
        text.parse().unwrap()
    }

    /// A `save` that records what it saved.
    fn saving(saved: &RefCell<Option<String>>) -> impl FnOnce(&str) -> Result<(), ()> + '_ {
        |text| {
            *saved.borrow_mut() = Some(text.to_owned());
            Ok(())
        }
    }

    #[test]
    fn the_default_shortcut_is_valid() {
        assert!(parse(super::DEFAULT_SHORTCUT).is_ok());
    }

    #[test]
    fn a_shortcut_needs_ctrl_alt_or_win() {
        assert!(parse("Alt+R").is_ok());
        assert!(parse("Super+Shift+K").is_ok());
        assert_eq!(parse("Shift+A"), Err(ShortcutError::Invalid));
        assert_eq!(parse("Space"), Err(ShortcutError::Invalid));
    }

    /// Settings records a press by its physical key, with the key code's
    /// name, so these must parse to the keys pressed.
    #[test]
    fn a_shortcut_recorded_by_physical_key_is_valid() {
        use tauri_plugin_global_shortcut::{Code, Modifiers};

        let recorded = [
            (
                "Ctrl+Shift+Space",
                Modifiers::CONTROL | Modifiers::SHIFT,
                Code::Space,
            ),
            (
                "Ctrl+Alt+K",
                Modifiers::CONTROL | Modifiers::ALT,
                Code::KeyK,
            ),
            ("Alt+7", Modifiers::ALT, Code::Digit7),
            ("Ctrl+Minus", Modifiers::CONTROL, Code::Minus),
            ("Ctrl+BracketLeft", Modifiers::CONTROL, Code::BracketLeft),
            ("Alt+ArrowUp", Modifiers::ALT, Code::ArrowUp),
            ("Ctrl+Numpad1", Modifiers::CONTROL, Code::Numpad1),
            (
                "Ctrl+Shift+F5",
                Modifiers::CONTROL | Modifiers::SHIFT,
                Code::F5,
            ),
        ];
        for (text, mods, key) in recorded {
            assert_eq!(parse(text), Ok(Shortcut::new(Some(mods), key)), "{text}");
        }
    }

    #[test]
    fn text_that_is_not_a_shortcut_is_invalid() {
        assert_eq!(parse("Ctrl+Shift+"), Err(ShortcutError::Invalid));
        assert_eq!(parse("Ctrl+Banana"), Err(ShortcutError::Invalid));
        assert_eq!(parse(""), Err(ShortcutError::Invalid));
    }

    #[test]
    fn a_new_shortcut_replaces_the_old_one_and_is_saved() {
        let hotkeys = FakeHotkeys::with("Ctrl+Shift+Space");
        let saved = RefCell::new(None);

        change(&hotkeys, "Ctrl+Shift+Space", "Alt+Shift+R", saving(&saved)).unwrap();

        assert!(hotkeys.works("Alt+Shift+R"));
        assert!(!hotkeys.works("Ctrl+Shift+Space"));
        assert_eq!(saved.into_inner().as_deref(), Some("Alt+Shift+R"));
    }

    #[test]
    fn a_shortcut_held_by_another_app_is_refused_and_the_old_one_keeps_working() {
        let hotkeys = FakeHotkeys::with("Ctrl+Shift+Space").taken("Alt+Shift+R");
        let saved = RefCell::new(None);

        let result = change(&hotkeys, "Ctrl+Shift+Space", "Alt+Shift+R", saving(&saved));

        assert_eq!(result, Err(ChangeError::Shortcut(ShortcutError::Taken)));
        assert!(hotkeys.works("Ctrl+Shift+Space"));
        assert_eq!(hotkeys.count(), 1);
        assert_eq!(saved.into_inner(), None);
    }

    #[test]
    fn an_invalid_shortcut_is_refused_and_the_old_one_keeps_working() {
        let hotkeys = FakeHotkeys::with("Ctrl+Shift+Space");
        let saved = RefCell::new(None);

        let result = change(&hotkeys, "Ctrl+Shift+Space", "Shift+A", saving(&saved));

        assert_eq!(result, Err(ChangeError::Shortcut(ShortcutError::Invalid)));
        assert!(hotkeys.works("Ctrl+Shift+Space"));
        assert_eq!(saved.into_inner(), None);
    }

    #[test]
    fn a_shortcut_that_cannot_be_saved_is_dropped_and_the_old_one_keeps_working() {
        let hotkeys = FakeHotkeys::with("Ctrl+Shift+Space");

        let result = change(&hotkeys, "Ctrl+Shift+Space", "Alt+Shift+R", |_| Err(()));

        assert_eq!(result, Err(ChangeError::Save));
        assert!(hotkeys.works("Ctrl+Shift+Space"));
        assert_eq!(hotkeys.count(), 1);
    }

    #[test]
    fn picking_the_shortcut_already_in_use_changes_nothing() {
        let hotkeys = FakeHotkeys::with("Ctrl+Shift+Space");
        let saved = RefCell::new(None);

        change(
            &hotkeys,
            "Ctrl+Shift+Space",
            "ctrl+shift+space",
            saving(&saved),
        )
        .unwrap();

        assert!(hotkeys.works("Ctrl+Shift+Space"));
        assert_eq!(saved.into_inner(), None);
    }

    #[test]
    fn picking_a_shortcut_that_failed_at_startup_tries_it_again() {
        let hotkeys = FakeHotkeys::default();
        let saved = RefCell::new(None);

        change(
            &hotkeys,
            "Ctrl+Shift+Space",
            "Ctrl+Shift+Space",
            saving(&saved),
        )
        .unwrap();

        assert!(hotkeys.works("Ctrl+Shift+Space"));
        assert_eq!(saved.into_inner().as_deref(), Some("Ctrl+Shift+Space"));
    }

    #[test]
    fn a_new_shortcut_works_even_if_the_saved_one_never_did() {
        let hotkeys = FakeHotkeys::default();
        let saved = RefCell::new(None);

        change(&hotkeys, "Ctrl+Banana", "Alt+Shift+R", saving(&saved)).unwrap();

        assert!(hotkeys.works("Alt+Shift+R"));
        assert_eq!(saved.into_inner().as_deref(), Some("Alt+Shift+R"));
    }

    #[test]
    fn resetting_to_the_default_brings_it_back_and_saves_it() {
        let hotkeys = FakeHotkeys::with("Alt+Shift+R");
        let saved = RefCell::new(None);

        change(
            &hotkeys,
            "Alt+Shift+R",
            super::DEFAULT_SHORTCUT,
            saving(&saved),
        )
        .unwrap();

        assert!(hotkeys.works("Ctrl+Shift+Space"));
        assert!(!hotkeys.works("Alt+Shift+R"));
        assert!(saved.into_inner().is_some_and(|text| is_default(&text)));
    }

    #[test]
    fn resetting_to_a_default_held_by_another_app_keeps_the_current_shortcut() {
        let hotkeys = FakeHotkeys::with("Alt+Shift+R").taken("Ctrl+Shift+Space");
        let saved = RefCell::new(None);

        let result = change(
            &hotkeys,
            "Alt+Shift+R",
            super::DEFAULT_SHORTCUT,
            saving(&saved),
        );

        assert_eq!(result, Err(ChangeError::Shortcut(ShortcutError::Taken)));
        assert!(hotkeys.works("Alt+Shift+R"));
        assert_eq!(hotkeys.count(), 1);
        assert_eq!(saved.into_inner(), None);
    }

    #[test]
    fn the_default_is_recognised_however_it_is_written() {
        assert!(is_default("Ctrl+Shift+Space"));
        assert!(is_default("shift+control+space"));
        assert!(!is_default("Alt+Shift+R"));
        assert!(!is_default("Ctrl+Banana"));
    }

    #[test]
    fn pausing_frees_the_shortcut() {
        let hotkeys = FakeHotkeys::with("Ctrl+Shift+Space");

        pause(&hotkeys, "Ctrl+Shift+Space");

        assert!(!hotkeys.works("Ctrl+Shift+Space"));
        assert_eq!(hotkeys.count(), 0);
    }

    #[test]
    fn resuming_brings_the_paused_shortcut_back_and_twice_is_harmless() {
        let hotkeys = FakeHotkeys::with("Ctrl+Shift+Space");
        pause(&hotkeys, "Ctrl+Shift+Space");

        resume(&hotkeys, "Ctrl+Shift+Space").unwrap();
        resume(&hotkeys, "Ctrl+Shift+Space").unwrap();

        assert!(hotkeys.works("Ctrl+Shift+Space"));
        assert_eq!(hotkeys.count(), 1);
    }

    #[test]
    fn recording_the_paused_shortcut_again_saves_it_and_leaves_it_working() {
        let hotkeys = FakeHotkeys::with("Ctrl+Shift+Space");
        let saved = RefCell::new(None);
        pause(&hotkeys, "Ctrl+Shift+Space");

        change(
            &hotkeys,
            "Ctrl+Shift+Space",
            "Ctrl+Shift+Space",
            saving(&saved),
        )
        .unwrap();
        resume(&hotkeys, "Ctrl+Shift+Space").unwrap();

        assert!(hotkeys.works("Ctrl+Shift+Space"));
        assert_eq!(hotkeys.count(), 1);
        assert_eq!(saved.into_inner().as_deref(), Some("Ctrl+Shift+Space"));
    }

    #[test]
    fn a_new_shortcut_picked_while_paused_works_and_is_saved() {
        let hotkeys = FakeHotkeys::with("Ctrl+Shift+Space");
        let saved = RefCell::new(None);
        pause(&hotkeys, "Ctrl+Shift+Space");

        change(&hotkeys, "Ctrl+Shift+Space", "Alt+Shift+R", saving(&saved)).unwrap();
        resume(&hotkeys, "Alt+Shift+R").unwrap();

        assert!(hotkeys.works("Alt+Shift+R"));
        assert!(!hotkeys.works("Ctrl+Shift+Space"));
        assert_eq!(saved.into_inner().as_deref(), Some("Alt+Shift+R"));
    }

    #[test]
    fn a_shortcut_picked_while_paused_that_cannot_be_saved_is_dropped_and_resuming_brings_the_old_one_back()
     {
        let hotkeys = FakeHotkeys::with("Ctrl+Shift+Space");
        pause(&hotkeys, "Ctrl+Shift+Space");

        let result = change(&hotkeys, "Ctrl+Shift+Space", "Alt+Shift+R", |_| Err(()));
        resume(&hotkeys, "Ctrl+Shift+Space").unwrap();

        assert_eq!(result, Err(ChangeError::Save));
        assert!(hotkeys.works("Ctrl+Shift+Space"));
        assert_eq!(hotkeys.count(), 1);
    }
}
