//! Global shortcut: registers the rewrite shortcut, changes it, and reports
//! each press.
//!
//! The `global-shortcut` plugin is registered in [`crate::run`].

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};

use tauri::AppHandle;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

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

/// Why a new shortcut can't be picked. A saved one is loaded by the older,
/// looser rules of [`parse`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Refusal {
    /// The text isn't a shortcut at all.
    #[error("the text is not a shortcut")]
    NotAShortcut,
    /// It uses Win, which Windows keeps for itself.
    #[error("the shortcut uses Win")]
    Win,
    /// It has neither Ctrl nor Alt.
    #[error("the shortcut has neither Ctrl nor Alt")]
    NoCtrlOrAlt,
    /// It is Ctrl+C or Ctrl+V, which the app presses to copy the selection
    /// and paste the result.
    #[error("the app needs the shortcut to copy and paste")]
    CopyPaste,
}

/// The shortcut couldn't be changed. The previous one still works.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ChangeError {
    /// The new shortcut breaks a rule for picking one.
    #[error(transparent)]
    Refused(#[from] Refusal),
    /// Windows refused it, almost always because another app uses it.
    #[error("the shortcut is used by another app")]
    Taken,
    /// The new shortcut worked but couldn't be saved, so it was dropped.
    #[error("the shortcut couldn't be saved")]
    Save,
}

/// Parses `text` into a shortcut the user may pick now. Stricter than
/// [`parse`], so a shortcut saved under the old rules keeps working until the
/// user changes it.
///
/// # Errors
///
/// Returns the [`Refusal`] that says why `text` can't be picked.
pub fn parse_new(text: &str) -> Result<Shortcut, Refusal> {
    let shortcut: Shortcut = text.trim().parse().map_err(|_| Refusal::NotAShortcut)?;
    if shortcut.mods.contains(Modifiers::SUPER) {
        return Err(Refusal::Win);
    }
    if !shortcut
        .mods
        .intersects(Modifiers::CONTROL | Modifiers::ALT)
    {
        return Err(Refusal::NoCtrlOrAlt);
    }
    let copy_or_paste = [Code::KeyC, Code::KeyV]
        .map(|key| Shortcut::new(Some(Modifiers::CONTROL), key))
        .contains(&shortcut);
    if copy_or_paste {
        return Err(Refusal::CopyPaste);
    }
    Ok(shortcut)
}

/// Parses `text` into a shortcut the app loads: Ctrl, Alt or Win, any
/// Shift, and one key. Shift alone isn't enough, as it would take over a
/// capital letter or symbol while the user types. These are the rules a
/// saved shortcut was picked under; a new one must pass [`parse_new`].
///
/// # Errors
///
/// Returns [`ShortcutError::Invalid`] if `text` isn't such a shortcut.
pub fn parse(text: &str) -> Result<Shortcut, ShortcutError> {
    let shortcut: Shortcut = text.trim().parse().map_err(|_| ShortcutError::Invalid)?;
    let required = Modifiers::CONTROL | Modifiers::ALT | Modifiers::SUPER;
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
/// `new` must pass the rules for a new shortcut ([`parse_new`]); `current` is
/// read by the looser rules a saved one was picked under ([`parse`]).
///
/// # Errors
///
/// Returns [`ChangeError::Refused`] if `new` breaks a rule,
/// [`ChangeError::Taken`] if Windows refuses it and [`ChangeError::Save`] if
/// saving fails. `current` stays registered then.
pub fn change<E>(
    hotkeys: &impl Hotkeys,
    current: &str,
    new: &str,
    save: impl FnOnce(&str) -> Result<(), E>,
) -> Result<(), ChangeError> {
    let new_shortcut = parse_new(new)?;
    let current_shortcut = parse(current).ok();
    let unchanged = current_shortcut == Some(new_shortcut);
    if unchanged && hotkeys.is_registered(new_shortcut) {
        return Ok(());
    }
    hotkeys
        .register(new_shortcut)
        .map_err(|_| ChangeError::Taken)?;
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

/// Pauses the saved shortcut while Settings records a new one, so pressing it
/// reaches the field instead of starting a rewrite.
///
/// The shortcut is paused exactly while two things hold: Settings asked for
/// it ([`Pause::request`]) and the Settings window has focus
/// ([`Pause::set_focused`], [`Pause::close`]). Every call settles the
/// shortcut to match both, so the order the calls arrive in doesn't matter:
/// a late pause can't leave the app without a shortcut once Settings has lost
/// focus or closed, and a late resume can't free the field while it records.
///
/// Every change, pause and resume takes the same lock, so two quick changes
/// can't both read the same previous shortcut and a resume can't bring back a
/// shortcut that a change is replacing. The saved shortcut is read through
/// `saved` under that lock for the same reason.
#[derive(Debug, Default)]
pub struct Pause {
    /// Settings asked to pause and hasn't asked to resume since.
    requested: AtomicBool,
    /// The Settings window has focus.
    focused: AtomicBool,
    /// The lock. Holds whether a working shortcut is paused, so a failure to
    /// bring it back is reported once, and a shortcut that never worked
    /// isn't reported again.
    paused: Mutex<bool>,
}

impl Pause {
    /// Nothing paused, and Settings not focused.
    pub const fn new() -> Self {
        Self {
            requested: AtomicBool::new(false),
            focused: AtomicBool::new(false),
            paused: Mutex::new(false),
        }
    }

    /// Settings starts (`recording`) or stops recording a new shortcut.
    ///
    /// # Errors
    ///
    /// Returns [`ShortcutError::Taken`] if a paused shortcut can't come back,
    /// because another app took it meanwhile.
    pub fn request(
        &self,
        hotkeys: &impl Hotkeys,
        saved: impl FnOnce() -> String,
        recording: bool,
    ) -> Result<(), ShortcutError> {
        let mut paused = self.lock();
        self.requested.store(recording, Ordering::SeqCst);
        self.settle_locked(hotkeys, &saved(), &mut paused)
    }

    /// The Settings window gained or lost focus. Never blocks, so window
    /// events can call it; call [`Pause::settle`] after it.
    pub fn set_focused(&self, focused: bool) {
        self.focused.store(focused, Ordering::SeqCst);
    }

    /// The Settings window closed, which ends any recording. Never blocks;
    /// call [`Pause::settle`] after it.
    pub fn close(&self) {
        self.requested.store(false, Ordering::SeqCst);
        self.focused.store(false, Ordering::SeqCst);
    }

    /// Pauses or brings back the saved shortcut to match the last request
    /// and focus. Does nothing if `saved` isn't a valid shortcut.
    ///
    /// # Errors
    ///
    /// Returns [`ShortcutError::Taken`] if a paused shortcut can't come back,
    /// because another app took it meanwhile.
    pub fn settle(
        &self,
        hotkeys: &impl Hotkeys,
        saved: impl FnOnce() -> String,
    ) -> Result<(), ShortcutError> {
        let mut paused = self.lock();
        self.settle_locked(hotkeys, &saved(), &mut paused)
    }

    /// [`change`] under the lock. If Settings is still recording, the new
    /// shortcut is paused too, so pressing it again records it instead of
    /// starting a rewrite.
    ///
    /// # Errors
    ///
    /// As [`change`].
    pub fn change<E>(
        &self,
        hotkeys: &impl Hotkeys,
        saved: impl FnOnce() -> String,
        new: &str,
        save: impl FnOnce(&str) -> Result<(), E>,
    ) -> Result<(), ChangeError> {
        let mut paused = self.lock();
        let changed = change(hotkeys, &saved(), new, save);
        if changed.is_ok() && self.wants_pause() {
            // Only pausing: a resume here would report a failure that the
            // next settle reports anyway.
            if let Ok(new) = parse(new) {
                hotkeys.unregister(new);
                *paused = true;
            }
        }
        changed
    }

    fn lock(&self) -> MutexGuard<'_, bool> {
        self.paused.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn wants_pause(&self) -> bool {
        self.requested.load(Ordering::SeqCst) && self.focused.load(Ordering::SeqCst)
    }

    fn settle_locked(
        &self,
        hotkeys: &impl Hotkeys,
        saved: &str,
        paused: &mut bool,
    ) -> Result<(), ShortcutError> {
        let Ok(saved) = parse(saved) else {
            return Ok(());
        };
        let registered = hotkeys.is_registered(saved);
        if self.wants_pause() {
            if registered {
                hotkeys.unregister(saved);
                *paused = true;
            }
            return Ok(());
        }
        let was_paused = std::mem::take(paused);
        if registered {
            return Ok(());
        }
        // Tried even if it never worked, in case the app holding it let go.
        match hotkeys.register(saved) {
            Err(error) if was_paused => Err(error),
            _ => Ok(()),
        }
    }
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

    use super::{ChangeError, Hotkeys, Pause, Refusal, ShortcutError, change, is_default, parse};

    /// Global shortcuts in memory. `taken` are held by "another app".
    #[derive(Default)]
    struct FakeHotkeys {
        taken: RefCell<HashSet<Shortcut>>,
        registered: RefCell<HashSet<Shortcut>>,
    }

    impl FakeHotkeys {
        fn with(registered: &str) -> Self {
            let hotkeys = Self::default();
            hotkeys.registered.borrow_mut().insert(shortcut(registered));
            hotkeys
        }

        fn taken(self, taken: &str) -> Self {
            self.take(taken);
            self
        }

        /// Another app registers `taken` now, as if it grabbed it while paused.
        fn take(&self, taken: &str) {
            self.taken.borrow_mut().insert(shortcut(taken));
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
            if self.taken.borrow().contains(&shortcut)
                || !self.registered.borrow_mut().insert(shortcut)
            {
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

    /// Reads the saved shortcut as the settings window would.
    fn saved(text: &str) -> impl FnOnce() -> String + '_ {
        || text.to_owned()
    }

    /// Settings is open and has focus, and its field records over `text`.
    fn recording(hotkeys: &FakeHotkeys, text: &str) -> Pause {
        let pause = Pause::new();
        pause.set_focused(true);
        pause.request(hotkeys, saved(text), true).unwrap();
        pause
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
    fn a_saved_shortcut_loads_with_ctrl_alt_or_win() {
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

        assert_eq!(result, Err(ChangeError::Taken));
        assert!(hotkeys.works("Ctrl+Shift+Space"));
        assert_eq!(hotkeys.count(), 1);
        assert_eq!(saved.into_inner(), None);
    }

    #[test]
    fn a_new_shortcut_without_ctrl_or_alt_is_refused_and_the_old_one_keeps_working() {
        let hotkeys = FakeHotkeys::with("Ctrl+Shift+Space");
        let saved = RefCell::new(None);

        let result = change(&hotkeys, "Ctrl+Shift+Space", "Shift+A", saving(&saved));

        assert_eq!(result, Err(ChangeError::Refused(Refusal::NoCtrlOrAlt)));
        assert!(hotkeys.works("Ctrl+Shift+Space"));
        assert_eq!(saved.into_inner(), None);
    }

    #[test]
    fn a_new_shortcut_with_win_is_refused_and_the_old_one_keeps_working() {
        let hotkeys = FakeHotkeys::with("Ctrl+Shift+Space");
        let saved = RefCell::new(None);

        let result = change(&hotkeys, "Ctrl+Shift+Space", "Ctrl+Super+K", saving(&saved));

        assert_eq!(result, Err(ChangeError::Refused(Refusal::Win)));
        assert!(hotkeys.works("Ctrl+Shift+Space"));
        assert_eq!(hotkeys.count(), 1);
        assert_eq!(saved.into_inner(), None);
    }

    /// The app presses Ctrl+C to copy the selection and Ctrl+V to paste the
    /// result, so taking either would start a rewrite inside a rewrite.
    #[test]
    fn ctrl_c_and_ctrl_v_are_refused_and_the_old_one_keeps_working() {
        for copy_or_paste in ["Ctrl+C", "control+v"] {
            let hotkeys = FakeHotkeys::with("Ctrl+Shift+Space");
            let saved = RefCell::new(None);

            let result = change(&hotkeys, "Ctrl+Shift+Space", copy_or_paste, saving(&saved));

            assert_eq!(
                result,
                Err(ChangeError::Refused(Refusal::CopyPaste)),
                "{copy_or_paste}"
            );
            assert!(hotkeys.works("Ctrl+Shift+Space"));
            assert_eq!(hotkeys.count(), 1);
            assert_eq!(saved.into_inner(), None);
        }
    }

    #[test]
    fn a_new_shortcut_with_ctrl_or_alt_and_any_shift_is_allowed() {
        for allowed in [
            "Ctrl+K",
            "Alt+K",
            "Ctrl+Shift+K",
            "Alt+Shift+K",
            "Ctrl+Alt+K",
        ] {
            let hotkeys = FakeHotkeys::with("Ctrl+Shift+Space");
            let saved = RefCell::new(None);

            change(&hotkeys, "Ctrl+Shift+Space", allowed, saving(&saved)).unwrap();

            assert!(hotkeys.works(allowed), "{allowed}");
            assert_eq!(saved.into_inner().as_deref(), Some(allowed));
        }
    }

    #[test]
    fn a_key_on_its_own_is_refused_for_missing_ctrl_or_alt() {
        let hotkeys = FakeHotkeys::with("Ctrl+Shift+Space");

        let result = change(&hotkeys, "Ctrl+Shift+Space", "K", |_| Ok::<_, ()>(()));

        assert_eq!(result, Err(ChangeError::Refused(Refusal::NoCtrlOrAlt)));
        assert!(hotkeys.works("Ctrl+Shift+Space"));
    }

    /// Copying or pasting with Shift held isn't what the app presses.
    #[test]
    fn ctrl_c_and_ctrl_v_with_more_modifiers_are_allowed() {
        let hotkeys = FakeHotkeys::with("Ctrl+Shift+Space");

        change(&hotkeys, "Ctrl+Shift+Space", "Ctrl+Shift+C", |_| {
            Ok::<_, ()>(())
        })
        .unwrap();

        assert!(hotkeys.works("Ctrl+Shift+C"));
    }

    #[test]
    fn text_that_is_not_a_shortcut_is_refused_as_not_a_shortcut() {
        for text in ["Ctrl+Banana", "Ctrl+Shift+", ""] {
            let hotkeys = FakeHotkeys::with("Ctrl+Shift+Space");

            let result = change(&hotkeys, "Ctrl+Shift+Space", text, |_| Ok::<_, ()>(()));

            assert_eq!(
                result,
                Err(ChangeError::Refused(Refusal::NotAShortcut)),
                "{text:?}"
            );
            assert!(hotkeys.works("Ctrl+Shift+Space"));
        }
    }

    /// Win+K was allowed before the recorder; it keeps working until the
    /// user picks another shortcut.
    #[test]
    fn a_saved_win_shortcut_still_loads_pauses_and_resumes() {
        let hotkeys = FakeHotkeys::with("Super+K");

        let pause = recording(&hotkeys, "Super+K");
        assert!(!hotkeys.works("Super+K"));
        pause.request(&hotkeys, saved("Super+K"), false).unwrap();

        assert!(hotkeys.works("Super+K"));
        assert!(!is_default("Super+K"));
    }

    #[test]
    fn a_saved_win_shortcut_is_replaced_by_a_new_one() {
        let hotkeys = FakeHotkeys::with("Super+K");
        let saved = RefCell::new(None);

        change(&hotkeys, "Super+K", "Alt+Shift+R", saving(&saved)).unwrap();

        assert!(hotkeys.works("Alt+Shift+R"));
        assert!(!hotkeys.works("Super+K"));
        assert_eq!(saved.into_inner().as_deref(), Some("Alt+Shift+R"));
    }

    #[test]
    fn a_saved_win_shortcut_cannot_be_picked_again() {
        let hotkeys = FakeHotkeys::with("Super+K");

        let result = change(&hotkeys, "Super+K", "Super+K", |_| Ok::<_, ()>(()));

        assert_eq!(result, Err(ChangeError::Refused(Refusal::Win)));
        assert!(hotkeys.works("Super+K"));
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

        assert_eq!(result, Err(ChangeError::Taken));
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
    fn recording_pauses_the_shortcut_while_settings_has_focus() {
        let hotkeys = FakeHotkeys::with("Ctrl+Shift+Space");

        recording(&hotkeys, "Ctrl+Shift+Space");

        assert!(!hotkeys.works("Ctrl+Shift+Space"));
        assert_eq!(hotkeys.count(), 0);
    }

    #[test]
    fn ending_recording_brings_the_paused_shortcut_back_and_twice_is_harmless() {
        let hotkeys = FakeHotkeys::with("Ctrl+Shift+Space");
        let pause = recording(&hotkeys, "Ctrl+Shift+Space");

        pause
            .request(&hotkeys, saved("Ctrl+Shift+Space"), false)
            .unwrap();
        pause
            .request(&hotkeys, saved("Ctrl+Shift+Space"), false)
            .unwrap();

        assert!(hotkeys.works("Ctrl+Shift+Space"));
        assert_eq!(hotkeys.count(), 1);
    }

    /// A pause sent while Settings had focus can reach Rust after Settings
    /// lost focus, when nothing would resume the shortcut any more.
    #[test]
    fn a_pause_that_arrives_after_settings_lost_focus_leaves_the_shortcut_working() {
        let hotkeys = FakeHotkeys::with("Ctrl+Shift+Space");
        let pause = Pause::new();
        pause.set_focused(true);
        pause.set_focused(false);
        pause.settle(&hotkeys, saved("Ctrl+Shift+Space")).unwrap();

        pause
            .request(&hotkeys, saved("Ctrl+Shift+Space"), true)
            .unwrap();

        assert!(hotkeys.works("Ctrl+Shift+Space"));
    }

    #[test]
    fn losing_focus_brings_the_paused_shortcut_back_without_a_resume() {
        let hotkeys = FakeHotkeys::with("Ctrl+Shift+Space");
        let pause = recording(&hotkeys, "Ctrl+Shift+Space");

        pause.set_focused(false);
        pause.settle(&hotkeys, saved("Ctrl+Shift+Space")).unwrap();

        assert!(hotkeys.works("Ctrl+Shift+Space"));
    }

    /// Settings closed mid-save: its last pause can arrive after the close.
    #[test]
    fn closing_settings_brings_the_paused_shortcut_back_and_a_later_pause_is_ignored() {
        let hotkeys = FakeHotkeys::with("Ctrl+Shift+Space");
        let pause = recording(&hotkeys, "Ctrl+Shift+Space");

        pause.close();
        pause.settle(&hotkeys, saved("Ctrl+Shift+Space")).unwrap();
        pause
            .request(&hotkeys, saved("Ctrl+Shift+Space"), true)
            .unwrap();

        assert!(hotkeys.works("Ctrl+Shift+Space"));
    }

    #[test]
    fn closing_settings_ends_recording_for_the_next_window() {
        let hotkeys = FakeHotkeys::with("Ctrl+Shift+Space");
        let pause = recording(&hotkeys, "Ctrl+Shift+Space");
        pause.close();
        pause.settle(&hotkeys, saved("Ctrl+Shift+Space")).unwrap();

        pause.set_focused(true);
        pause.settle(&hotkeys, saved("Ctrl+Shift+Space")).unwrap();

        assert!(hotkeys.works("Ctrl+Shift+Space"));
    }

    /// A field still recording when Settings regains focus pauses again,
    /// even if its pause reached Rust before the focus did.
    #[test]
    fn regaining_focus_while_recording_pauses_the_shortcut_again() {
        let hotkeys = FakeHotkeys::with("Ctrl+Shift+Space");
        let pause = Pause::new();
        pause
            .request(&hotkeys, saved("Ctrl+Shift+Space"), true)
            .unwrap();
        assert!(hotkeys.works("Ctrl+Shift+Space"));

        pause.set_focused(true);
        pause.settle(&hotkeys, saved("Ctrl+Shift+Space")).unwrap();

        assert!(!hotkeys.works("Ctrl+Shift+Space"));
    }

    #[test]
    fn a_paused_shortcut_another_app_took_is_reported_once_when_it_cannot_come_back() {
        let hotkeys = FakeHotkeys::with("Ctrl+Shift+Space");
        let pause = recording(&hotkeys, "Ctrl+Shift+Space");
        hotkeys.take("Ctrl+Shift+Space");

        let ended = pause.request(&hotkeys, saved("Ctrl+Shift+Space"), false);
        let again = pause.request(&hotkeys, saved("Ctrl+Shift+Space"), false);

        assert_eq!(ended, Err(ShortcutError::Taken));
        assert_eq!(again, Ok(()));
    }

    /// The app already reported it when it started.
    #[test]
    fn a_shortcut_that_never_worked_is_not_reported_when_recording_ends() {
        let hotkeys = FakeHotkeys::default().taken("Ctrl+Shift+Space");
        let pause = recording(&hotkeys, "Ctrl+Shift+Space");

        let ended = pause.request(&hotkeys, saved("Ctrl+Shift+Space"), false);

        assert_eq!(ended, Ok(()));
        assert!(!hotkeys.works("Ctrl+Shift+Space"));
    }

    #[test]
    fn recording_the_paused_shortcut_again_saves_it_and_it_works_once_recording_ends() {
        let hotkeys = FakeHotkeys::with("Ctrl+Shift+Space");
        let saved_text = RefCell::new(None);
        let pause = recording(&hotkeys, "Ctrl+Shift+Space");

        pause
            .change(
                &hotkeys,
                saved("Ctrl+Shift+Space"),
                "Ctrl+Shift+Space",
                saving(&saved_text),
            )
            .unwrap();
        assert!(!hotkeys.works("Ctrl+Shift+Space"), "still recording");
        pause
            .request(&hotkeys, saved("Ctrl+Shift+Space"), false)
            .unwrap();

        assert!(hotkeys.works("Ctrl+Shift+Space"));
        assert_eq!(hotkeys.count(), 1);
        assert_eq!(saved_text.into_inner().as_deref(), Some("Ctrl+Shift+Space"));
    }

    /// Pressing the new shortcut again while the field records records it,
    /// instead of starting a rewrite.
    #[test]
    fn a_new_shortcut_picked_while_recording_is_saved_and_works_once_recording_ends() {
        let hotkeys = FakeHotkeys::with("Ctrl+Shift+Space");
        let saved_text = RefCell::new(None);
        let pause = recording(&hotkeys, "Ctrl+Shift+Space");

        pause
            .change(
                &hotkeys,
                saved("Ctrl+Shift+Space"),
                "Alt+Shift+R",
                saving(&saved_text),
            )
            .unwrap();
        assert!(!hotkeys.works("Alt+Shift+R"), "still recording");
        pause
            .request(&hotkeys, saved("Alt+Shift+R"), false)
            .unwrap();

        assert!(hotkeys.works("Alt+Shift+R"));
        assert!(!hotkeys.works("Ctrl+Shift+Space"));
        assert_eq!(saved_text.into_inner().as_deref(), Some("Alt+Shift+R"));
    }

    /// As "Reset to default" does.
    #[test]
    fn a_new_shortcut_picked_while_not_recording_works_at_once() {
        let hotkeys = FakeHotkeys::with("Alt+Shift+R");
        let pause = Pause::new();
        pause.set_focused(true);

        pause
            .change(
                &hotkeys,
                saved("Alt+Shift+R"),
                super::DEFAULT_SHORTCUT,
                |_| Ok::<_, ()>(()),
            )
            .unwrap();

        assert!(hotkeys.works("Ctrl+Shift+Space"));
        assert!(!hotkeys.works("Alt+Shift+R"));
    }

    #[test]
    fn a_shortcut_picked_while_recording_that_cannot_be_saved_is_dropped_and_ending_recording_brings_the_old_one_back()
     {
        let hotkeys = FakeHotkeys::with("Ctrl+Shift+Space");
        let pause = recording(&hotkeys, "Ctrl+Shift+Space");

        let result = pause.change(&hotkeys, saved("Ctrl+Shift+Space"), "Alt+Shift+R", |_| {
            Err(())
        });
        pause
            .request(&hotkeys, saved("Ctrl+Shift+Space"), false)
            .unwrap();

        assert_eq!(result, Err(ChangeError::Save));
        assert!(hotkeys.works("Ctrl+Shift+Space"));
        assert_eq!(hotkeys.count(), 1);
    }
}
