//! The tray menu as plain data: which items it holds, in which order, which
//! instruction set is checked, and what a clicked item id means. No Tauri
//! here, so the rules are unit-tested; `tray` turns this into a real menu.

use crate::settings::{InstructionSetId, InstructionSets};

/// One entry of the tray menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrayItem {
    /// The "Instruction set" submenu.
    Submenu {
        /// Menu id.
        id: String,
        /// Label shown.
        label: String,
        /// One checkable item per instruction set, in library order.
        sets: Vec<SetItem>,
    },
    /// A plain clickable item.
    Item {
        /// Menu id.
        id: String,
        /// Label shown.
        label: String,
    },
    /// A separator line.
    Separator,
}

/// One instruction set in the submenu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetItem {
    /// Menu id; `action` maps it back to the set.
    pub id: String,
    /// The set's name, with `&` escaped so Windows shows it as typed.
    pub label: String,
    /// Whether this is the active instruction set.
    pub checked: bool,
}

/// Menu id of the "Instruction set" submenu.
const SETS_ID: &str = "sets";

/// Menu id of the "Settings" item.
const SETTINGS_ID: &str = "settings";

/// Menu id of the "Quit" item.
const QUIT_ID: &str = "quit";

/// Prefix of an instruction set item's menu id, followed by the set's id.
const SET_PREFIX: &str = "set:";

/// The tray menu for `sets`: the "Instruction set" submenu, then Settings,
/// a separator and Quit.
pub fn tray_menu(sets: &InstructionSets) -> Vec<TrayItem> {
    let active = sets.active().id;
    let set_items = sets
        .sets()
        .iter()
        .map(|set| SetItem {
            id: format!("{SET_PREFIX}{}", set.id),
            // A single `&` marks a keyboard shortcut in a Windows menu.
            label: set.name.replace('&', "&&"),
            checked: set.id == active,
        })
        .collect();
    vec![
        TrayItem::Submenu {
            id: SETS_ID.to_owned(),
            label: "Instruction set".to_owned(),
            sets: set_items,
        },
        TrayItem::Item {
            id: SETTINGS_ID.to_owned(),
            label: "Settings".to_owned(),
        },
        TrayItem::Separator,
        TrayItem::Item {
            id: QUIT_ID.to_owned(),
            label: "Quit".to_owned(),
        },
    ]
}

/// What a click on a tray menu item asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayAction {
    /// Make this instruction set the active one.
    Activate(InstructionSetId),
    /// Open the settings window.
    OpenSettings,
    /// Quit the app.
    Quit,
}

/// The action for the menu item with `id`, or `None` for ids that ask for
/// nothing (the submenu itself, unknown ids).
pub fn action(id: &str) -> Option<TrayAction> {
    match id {
        SETTINGS_ID => Some(TrayAction::OpenSettings),
        QUIT_ID => Some(TrayAction::Quit),
        _ => id
            .strip_prefix(SET_PREFIX)?
            .parse()
            .ok()
            .map(TrayAction::Activate),
    }
}

#[cfg(test)]
mod tests {
    use super::{TrayAction, TrayItem, action, tray_menu};
    use crate::settings::InstructionSets;

    fn labels(items: &[TrayItem]) -> Vec<&str> {
        items
            .iter()
            .map(|item| match item {
                TrayItem::Submenu { label, .. } | TrayItem::Item { label, .. } => label.as_str(),
                TrayItem::Separator => "---",
            })
            .collect()
    }

    fn submenu(items: &[TrayItem]) -> Vec<(&str, bool)> {
        match items.first() {
            Some(TrayItem::Submenu { sets, .. }) => sets
                .iter()
                .map(|set| (set.label.as_str(), set.checked))
                .collect(),
            other => panic!("first item is not the submenu: {other:?}"),
        }
    }

    #[test]
    fn the_menu_reads_instruction_set_then_settings_then_quit() {
        let menu = tray_menu(&InstructionSets::default());
        assert_eq!(
            labels(&menu),
            ["Instruction set", "Settings", "---", "Quit"]
        );
    }

    #[test]
    fn the_submenu_lists_every_set_in_order_with_the_active_one_checked() {
        let mut sets = InstructionSets::default();
        let mine = sets.add("Casual", "Make it casual.").expect("valid name");
        sets.set_active(mine).expect("set exists");

        assert_eq!(
            submenu(&tray_menu(&sets)),
            [
                ("Fix && make readable", false),
                ("Fix mistakes only", false),
                ("Professional", false),
                ("Shorter", false),
                ("Casual", true),
            ]
        );
    }

    #[test]
    fn clicking_a_set_item_activates_that_set() {
        let mut sets = InstructionSets::default();
        let mine = sets.add("Casual", "Make it casual.").expect("valid name");
        let menu = tray_menu(&sets);
        let Some(TrayItem::Submenu { sets: items, .. }) = menu.first() else {
            panic!("first item is not the submenu");
        };
        let clicked = items.last().expect("the submenu lists the sets");

        assert_eq!(action(&clicked.id), Some(TrayAction::Activate(mine)));
    }

    #[test]
    fn clicking_settings_or_quit_maps_to_that_action() {
        let menu = tray_menu(&InstructionSets::default());
        let actions: Vec<_> = menu
            .iter()
            .filter_map(|item| match item {
                TrayItem::Item { id, .. } => action(id),
                _ => None,
            })
            .collect();
        assert_eq!(actions, [TrayAction::OpenSettings, TrayAction::Quit]);
    }

    #[test]
    fn unknown_ids_do_nothing() {
        assert_eq!(action("sets"), None);
        assert_eq!(action("set:not-a-number"), None);
        assert_eq!(action("something"), None);
    }
}
