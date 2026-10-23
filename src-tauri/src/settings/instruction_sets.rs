//! The instruction set library: the user's ordered instruction sets and the
//! one active set. Pure rules, no I/O; it is saved as part of the settings.

use serde::{Deserialize, Serialize};

/// Identifies one instruction set. Never reused within a library, so a
/// stale id (from the tray or an old settings window) can't hit a newer set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct InstructionSetId(u64);

/// One of the starter templates that ship with the app. A set installed
/// from a template keeps this marker, so Restore can find it after the user
/// edits it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StarterTemplate {
    /// Fix mistakes and make the text easy to read. Active by default.
    FixAndMakeReadable,
    /// Fix mistakes, with no rewording.
    FixMistakesOnly,
    /// For emails and work messages.
    Professional,
    /// The same message in fewer words.
    Shorter,
}

impl StarterTemplate {
    /// Every starter template, in the order they are installed.
    pub const ALL: [Self; 4] = [
        Self::FixAndMakeReadable,
        Self::FixMistakesOnly,
        Self::Professional,
        Self::Shorter,
    ];

    /// The template's original name.
    pub fn name(self) -> &'static str {
        match self {
            Self::FixAndMakeReadable => "Fix & make readable",
            Self::FixMistakesOnly => "Fix mistakes only",
            Self::Professional => "Professional",
            Self::Shorter => "Shorter",
        }
    }

    /// The template's original instructions.
    pub fn instructions(self) -> &'static str {
        match self {
            Self::FixAndMakeReadable => FIX_AND_MAKE_READABLE,
            Self::FixMistakesOnly => FIX_MISTAKES_ONLY,
            Self::Professional => PROFESSIONAL,
            Self::Shorter => SHORTER,
        }
    }
}

// Starter wording. Each line is one behaviour the user can read and edit.
// The language line is the same in all four: a rewrite never translates.

const FIX_AND_MAKE_READABLE: &str = "\
Fix spelling, grammar and punctuation mistakes, and make the text easy to read.
- Split run-on sentences and smooth awkward wording, but keep the writer's meaning, tone and voice.
- Don't add or remove information.
- Write in the same language and script as the text, and keep any mix of languages. Never translate.
- Keep the formatting, such as line breaks, lists, emoji, names and links.";

const FIX_MISTAKES_ONLY: &str = "\
Fix only spelling, grammar and punctuation mistakes.
- Change nothing else: keep the wording, word order, tone and style, even where they could be better.
- If there are no mistakes, return the text unchanged.
- Write in the same language and script as the text, and keep any mix of languages. Never translate.
- Keep the formatting, such as line breaks, lists, emoji, names and links.";

const PROFESSIONAL: &str = "\
Rewrite the text as a clear, professional message for work, such as an email or a message to a colleague.
- Fix spelling, grammar and punctuation mistakes.
- Use a polite, confident tone. Replace slang and overly casual wording, but don't make it stiff or wordy.
- Keep every fact, request, name, date and number.
- Don't add a greeting, sign-off or subject line the text doesn't already have.
- Write in the same language and script as the text, and keep any mix of languages. Never translate.
- Keep the formatting, such as line breaks and lists.";

const SHORTER: &str = "\
Rewrite the text to say the same thing in fewer words.
- Cut filler, repetition and needless detail, and fix spelling, grammar and punctuation mistakes.
- Keep the meaning, the writer's tone and every fact, request, name, date and number the reader needs.
- Write in the same language and script as the text, and keep any mix of languages. Never translate.
- Keep the formatting, such as line breaks and lists, where it still fits.";

/// One named instruction set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstructionSet {
    /// Stable identifier.
    pub id: InstructionSetId,
    /// The name shown in Settings and the tray.
    pub name: String,
    /// What a rewrite does, sent ahead of the fixed rules.
    pub instructions: String,
    /// The starter template this set was installed from, or `None` for a
    /// set the user wrote.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub starter: Option<StarterTemplate>,
}

/// An instruction set rule was broken. Holds no set content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InstructionSetError {
    /// The name is empty or only spaces.
    #[error("the instruction set has no name")]
    BlankName,
    /// No set has this id.
    #[error("the instruction set doesn't exist")]
    NotFound,
    /// The last set can't be deleted, so the shortcut always has one.
    #[error("the last instruction set can't be deleted")]
    LastSet,
}

/// The ordered instruction sets plus the active one. Always holds at least
/// one set, and the active id always names one of them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", from = "SavedInstructionSets")]
pub struct InstructionSets {
    sets: Vec<InstructionSet>,
    active: InstructionSetId,
    next_id: u64,
}

impl Default for InstructionSets {
    /// The starter templates, with "Fix & make readable" active.
    fn default() -> Self {
        let mut library = Self {
            sets: Vec::new(),
            active: InstructionSetId(0),
            next_id: 0,
        };
        for starter in StarterTemplate::ALL {
            library.push(starter.name(), starter.instructions(), Some(starter));
        }
        library
    }
}

impl InstructionSets {
    /// Every set, in order.
    pub fn sets(&self) -> &[InstructionSet] {
        &self.sets
    }

    /// The active instruction set.
    pub fn active(&self) -> &InstructionSet {
        self.sets
            .iter()
            .find(|set| set.id == self.active)
            .unwrap_or(&self.sets[0])
    }

    /// Adds a set the user wrote at the end of the list. The name is
    /// trimmed.
    ///
    /// # Errors
    ///
    /// Returns [`InstructionSetError::BlankName`] for an empty name.
    pub fn add(
        &mut self,
        name: &str,
        instructions: &str,
    ) -> Result<InstructionSetId, InstructionSetError> {
        let name = checked_name(name)?;
        Ok(self.push(name, instructions, None))
    }

    /// Changes a set's name and instructions, keeping its place in the list.
    /// The name is trimmed.
    ///
    /// # Errors
    ///
    /// Returns [`InstructionSetError::BlankName`] for an empty name and
    /// [`InstructionSetError::NotFound`] if no set has `id`.
    pub fn edit(
        &mut self,
        id: InstructionSetId,
        name: &str,
        instructions: &str,
    ) -> Result<(), InstructionSetError> {
        let name = checked_name(name)?;
        let set = self
            .sets
            .iter_mut()
            .find(|set| set.id == id)
            .ok_or(InstructionSetError::NotFound)?;
        name.clone_into(&mut set.name);
        instructions.clone_into(&mut set.instructions);
        Ok(())
    }

    /// Makes the set with `id` the active one.
    ///
    /// # Errors
    ///
    /// Returns [`InstructionSetError::NotFound`] if no set has `id`.
    pub fn set_active(&mut self, id: InstructionSetId) -> Result<(), InstructionSetError> {
        self.position(id)?;
        self.active = id;
        Ok(())
    }

    /// Deletes the set with `id`. Deleting the active set makes the first
    /// remaining set active.
    ///
    /// # Errors
    ///
    /// Returns [`InstructionSetError::NotFound`] if no set has `id` and
    /// [`InstructionSetError::LastSet`] if it is the only set left.
    pub fn delete(&mut self, id: InstructionSetId) -> Result<(), InstructionSetError> {
        let index = self.position(id)?;
        if self.sets.len() == 1 {
            return Err(InstructionSetError::LastSet);
        }
        self.sets.remove(index);
        if self.active == id {
            self.active = self.sets[0].id;
        }
        Ok(())
    }

    /// Restore default templates: every starter template still in the list
    /// gets its original name and instructions back in place, and every
    /// missing one is re-added at the end. Sets the user wrote are never
    /// touched, and the active set stays active.
    pub fn restore_starter_templates(&mut self) {
        for starter in StarterTemplate::ALL {
            match self
                .sets
                .iter_mut()
                .find(|set| set.starter == Some(starter))
            {
                Some(set) => {
                    starter.name().clone_into(&mut set.name);
                    starter.instructions().clone_into(&mut set.instructions);
                }
                None => {
                    self.push(starter.name(), starter.instructions(), Some(starter));
                }
            }
        }
    }

    fn position(&self, id: InstructionSetId) -> Result<usize, InstructionSetError> {
        self.sets
            .iter()
            .position(|set| set.id == id)
            .ok_or(InstructionSetError::NotFound)
    }

    fn push(
        &mut self,
        name: &str,
        instructions: &str,
        starter: Option<StarterTemplate>,
    ) -> InstructionSetId {
        let id = InstructionSetId(self.next_id);
        self.next_id += 1;
        self.sets.push(InstructionSet {
            id,
            name: name.to_owned(),
            instructions: instructions.to_owned(),
            starter,
        });
        id
    }
}

fn checked_name(name: &str) -> Result<&str, InstructionSetError> {
    let name = name.trim();
    if name.is_empty() {
        Err(InstructionSetError::BlankName)
    } else {
        Ok(name)
    }
}

/// The library as saved in the settings file, before its rules are checked.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SavedInstructionSets {
    sets: Vec<InstructionSet>,
    active: InstructionSetId,
    #[serde(default)]
    next_id: u64,
}

impl From<SavedInstructionSets> for InstructionSets {
    /// Repairs a hand-edited or damaged library instead of failing: an empty
    /// list becomes the starter templates, an unknown active id falls back
    /// to the first set, and new ids start past every id in use.
    fn from(saved: SavedInstructionSets) -> Self {
        let Some(first) = saved.sets.first() else {
            return Self::default();
        };
        let active = if saved.sets.iter().any(|set| set.id == saved.active) {
            saved.active
        } else {
            first.id
        };
        let past_ids = saved
            .sets
            .iter()
            .map(|set| set.id.0.saturating_add(1))
            .max()
            .unwrap_or(0);
        Self {
            active,
            next_id: saved.next_id.max(past_ids),
            sets: saved.sets,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{InstructionSetError, InstructionSetId, InstructionSets, StarterTemplate};

    fn names(sets: &InstructionSets) -> Vec<&str> {
        sets.sets().iter().map(|set| set.name.as_str()).collect()
    }

    fn id_of(sets: &InstructionSets, name: &str) -> InstructionSetId {
        sets.sets().iter().find(|set| set.name == name).unwrap().id
    }

    #[test]
    fn the_starter_templates_are_installed_with_fix_and_make_readable_active() {
        let sets = InstructionSets::default();

        assert_eq!(
            names(&sets),
            [
                "Fix & make readable",
                "Fix mistakes only",
                "Professional",
                "Shorter"
            ]
        );
        assert_eq!(sets.active().name, "Fix & make readable");
        assert_eq!(
            sets.active().starter,
            Some(StarterTemplate::FixAndMakeReadable)
        );
    }

    #[test]
    fn a_new_set_goes_at_the_end_and_is_not_a_starter() {
        let mut sets = InstructionSets::default();

        let id = sets
            .add("Roman Urdu to English", "Translate to clean English.")
            .unwrap();

        let added = sets.sets().last().unwrap();
        assert_eq!(added.id, id);
        assert_eq!(added.name, "Roman Urdu to English");
        assert_eq!(added.instructions, "Translate to clean English.");
        assert_eq!(added.starter, None);
        assert_eq!(sets.active().name, "Fix & make readable");
    }

    #[test]
    fn a_set_with_a_blank_name_is_refused() {
        let mut sets = InstructionSets::default();

        assert_eq!(
            sets.add("   ", "Anything."),
            Err(InstructionSetError::BlankName)
        );
        assert_eq!(sets.sets().len(), 4);
    }

    #[test]
    fn editing_changes_the_name_and_instructions_in_place() {
        let mut sets = InstructionSets::default();
        let professional = id_of(&sets, "Professional");

        sets.edit(professional, " Work ", "Sound like me at work.")
            .unwrap();

        assert_eq!(names(&sets)[2], "Work");
        assert_eq!(sets.sets()[2].instructions, "Sound like me at work.");
    }

    #[test]
    fn editing_a_missing_set_is_refused() {
        let mut sets = InstructionSets::default();
        let shorter = id_of(&sets, "Shorter");
        sets.delete(shorter).unwrap();

        assert_eq!(
            sets.edit(shorter, "Shorter", "Fewer words."),
            Err(InstructionSetError::NotFound)
        );
    }

    #[test]
    fn set_active_makes_that_set_the_active_one() {
        let mut sets = InstructionSets::default();
        let shorter = id_of(&sets, "Shorter");

        sets.set_active(shorter).unwrap();

        assert_eq!(sets.active().name, "Shorter");
    }

    #[test]
    fn deleting_a_set_keeps_the_others_in_order() {
        let mut sets = InstructionSets::default();

        sets.delete(id_of(&sets, "Fix mistakes only")).unwrap();

        assert_eq!(
            names(&sets),
            ["Fix & make readable", "Professional", "Shorter"]
        );
        assert_eq!(sets.active().name, "Fix & make readable");
    }

    #[test]
    fn deleting_the_active_set_makes_the_first_remaining_set_active() {
        let mut sets = InstructionSets::default();
        let first = id_of(&sets, "Fix & make readable");
        sets.set_active(id_of(&sets, "Professional")).unwrap();
        sets.delete(first).unwrap();

        sets.delete(id_of(&sets, "Professional")).unwrap();

        assert_eq!(sets.active().name, "Fix mistakes only");
    }

    #[test]
    fn the_last_set_cannot_be_deleted() {
        let mut sets = InstructionSets::default();
        for name in ["Fix & make readable", "Fix mistakes only", "Professional"] {
            sets.delete(id_of(&sets, name)).unwrap();
        }

        let result = sets.delete(id_of(&sets, "Shorter"));

        assert_eq!(result, Err(InstructionSetError::LastSet));
        assert_eq!(names(&sets), ["Shorter"]);
        assert_eq!(sets.active().name, "Shorter");
    }

    #[test]
    fn restore_re_adds_a_missing_starter_at_the_end() {
        let mut sets = InstructionSets::default();
        sets.delete(id_of(&sets, "Fix mistakes only")).unwrap();
        sets.add("Mine", "My rules.").unwrap();

        sets.restore_starter_templates();

        assert_eq!(
            names(&sets),
            [
                "Fix & make readable",
                "Professional",
                "Shorter",
                "Mine",
                "Fix mistakes only"
            ]
        );
        let restored = sets.sets().last().unwrap();
        assert_eq!(restored.starter, Some(StarterTemplate::FixMistakesOnly));
        assert_eq!(
            restored.instructions,
            StarterTemplate::FixMistakesOnly.instructions()
        );
    }

    #[test]
    fn restore_resets_an_edited_starter_in_place() {
        let mut sets = InstructionSets::default();
        let professional = id_of(&sets, "Professional");
        sets.edit(professional, "Work", "Sound like me at work.")
            .unwrap();

        sets.restore_starter_templates();

        assert_eq!(names(&sets)[2], "Professional");
        assert_eq!(sets.sets()[2].id, professional);
        assert_eq!(
            sets.sets()[2].instructions,
            StarterTemplate::Professional.instructions()
        );
        assert_eq!(sets.sets().len(), 4);
    }

    #[test]
    fn restore_never_touches_a_set_the_user_wrote() {
        let mut sets = InstructionSets::default();
        let mine = sets.add("Professional", "My own work rules.").unwrap();

        sets.restore_starter_templates();

        let kept = sets.sets().iter().find(|set| set.id == mine).unwrap();
        assert_eq!(kept.name, "Professional");
        assert_eq!(kept.instructions, "My own work rules.");
        assert_eq!(kept.starter, None);
        assert_eq!(sets.sets().len(), 5);
    }

    #[test]
    fn restore_keeps_the_active_set_active() {
        let mut sets = InstructionSets::default();
        let shorter = id_of(&sets, "Shorter");
        sets.edit(shorter, "Tiny", "As short as possible.").unwrap();
        sets.set_active(shorter).unwrap();
        sets.delete(id_of(&sets, "Fix & make readable")).unwrap();

        sets.restore_starter_templates();

        assert_eq!(sets.active().id, shorter);
        assert_eq!(sets.active().name, "Shorter");
    }

    #[test]
    fn restore_keeps_a_set_the_user_wrote_active() {
        let mut sets = InstructionSets::default();
        let mine = sets.add("Mine", "My rules.").unwrap();
        sets.set_active(mine).unwrap();

        sets.restore_starter_templates();

        assert_eq!(sets.active().id, mine);
    }

    #[test]
    fn a_saved_library_loads_back_unchanged() {
        let mut sets = InstructionSets::default();
        let mine = sets.add("Mine", "My rules.").unwrap();
        sets.set_active(mine).unwrap();

        let json = serde_json::to_string(&sets).unwrap();
        let loaded: InstructionSets = serde_json::from_str(&json).unwrap();

        assert_eq!(loaded, sets);
    }

    #[test]
    fn a_saved_empty_library_loads_as_the_starter_templates() {
        let loaded: InstructionSets =
            serde_json::from_str(r#"{ "sets": [], "active": 3, "nextId": 9 }"#).unwrap();

        assert_eq!(names(&loaded).len(), 4);
        assert_eq!(loaded.active().name, "Fix & make readable");
    }

    #[test]
    fn a_saved_active_id_that_names_no_set_loads_with_the_first_set_active() {
        let loaded: InstructionSets = serde_json::from_str(
            r#"{ "sets": [{ "id": 4, "name": "A", "instructions": "a" },
                          { "id": 7, "name": "B", "instructions": "b" }],
                 "active": 99, "nextId": 8 }"#,
        )
        .unwrap();

        assert_eq!(loaded.active().name, "A");
    }

    #[test]
    fn a_loaded_library_never_hands_out_an_id_already_in_use() {
        let mut loaded: InstructionSets = serde_json::from_str(
            r#"{ "sets": [{ "id": 0, "name": "B", "instructions": "b" }], "active": 0 }"#,
        )
        .unwrap();

        let added = loaded.add("New", "n").unwrap();

        assert_ne!(added, id_of(&loaded, "B"));
    }

    #[test]
    fn ids_of_deleted_sets_are_never_reused() {
        let mut sets = InstructionSets::default();
        let mine = sets.add("Mine", "My rules.").unwrap();
        sets.delete(mine).unwrap();

        let next = sets.add("Next", "Other rules.").unwrap();

        assert_ne!(next, mine);
    }
}
