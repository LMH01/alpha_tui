use std::{collections::HashMap, fmt::Display};

use anyhow::{anyhow, Result};
use ratatui::{
    text::{Line, Span},
    widgets::Paragraph,
};

use super::{ui::style::SharedTheme, State};

/// Manages all keybinding hints.
pub struct KeybindingHints {
    hints: HashMap<String, KeybindingHint>,
    theme: SharedTheme,
}

impl KeybindingHints {
    pub fn new(theme: SharedTheme) -> Result<Self> {
        Ok(Self {
            hints: default_keybindings()?,
            theme,
        })
    }

    /// Returns the keybinding hint paragraph ready to be printed.
    ///
    /// `width` is used to determine how many keybinding hints can be printed in one line.
    ///
    /// Return value `u16` is the amount of lines that this paragraph contains.
    pub fn keybinding_hint_paragraph(&self, width: u16) -> (Paragraph, u16) {
        let mut active_hints = self.active_keybinds();
        active_hints.sort_by_key(|f| f.order());
        let mut styled_keybinds_row = Vec::new();
        let mut styled_keybinds = Vec::new();
        let mut first_hint = true;
        let mut line_length = 0;
        for hint in &active_hints {
            if first_hint {
                first_hint = false;
            } else {
                styled_keybinds_row
                    .push(Span::from(" ").style(self.theme.keybinding_hint_paragraph()));
                line_length += 1;
            }
            let text = format!("{} [{}]", hint.label(), hint.key);
            let remaining_space = usize::saturating_sub(width as usize, text.len());
            if line_length > remaining_space {
                line_length = 0;
                styled_keybinds.push(Line::from(styled_keybinds_row));
                styled_keybinds_row = Vec::new();
            }
            line_length += text.len();
            styled_keybinds_row
                .push(Span::from(text).style(self.theme.keybinding_hints(hint.enabled)));
        }
        styled_keybinds.push(Line::from(styled_keybinds_row));

        let hight = styled_keybinds.len() as u16;
        let keybinding_hints =
            Paragraph::new(styled_keybinds).style(self.theme.keybinding_hint_paragraph());
        (keybinding_hints, hight)
    }

    /// Returns a list of keybinding hints that are currently active.
    fn active_keybinds(&self) -> Vec<KeybindingHint> {
        let mut hints = Vec::new();
        for hint in self.hints.values() {
            if hint.shown {
                hints.push(hint.clone());
            }
        }
        hints
    }

    /// Use to enable a keybinding hint.
    ///
    /// Does nothing if the key is not associated to a keybinding.
    fn enable(&mut self, key: &str) {
        if let Some(bind) = self.hints.get_mut(key) {
            bind.enabled = true;
        }
    }

    /// Use to disable a keybinding hint.
    ///
    /// Disabled keybinding hints are still displayed but grayed out.
    ///
    /// Does nothing if the key is not associated to a keybinding.
    fn disable(&mut self, key: &str) {
        if let Some(bind) = self.hints.get_mut(key) {
            bind.enabled = false;
        }
    }

    /// Use to show a keybinding hint.
    ///
    /// Does nothing if the key is not associated to a binding.
    fn show(&mut self, key: &str) {
        if let Some(bind) = self.hints.get_mut(key) {
            bind.shown = true;
        }
    }

    /// Use to hide a keybinding hint.
    ///
    /// Does nothing if the key is not associated to a binding.
    fn hide(&mut self, key: &str) {
        if let Some(bind) = self.hints.get_mut(key) {
            bind.shown = false;
        }
    }

    /// Use to enable and show a keybinding hint.
    fn show_and_enable(&mut self, key: &str) {
        if let Some(bind) = self.hints.get_mut(key) {
            bind.shown = true;
            bind.enabled = true;
        }
    }

    /// Checks the status of the keybinding hint.
    ///
    /// Returns `true` if the keybinding hint is shown or `false` if the keybinding hint is hidden or was not found.
    fn _status(&self, key: &str) -> bool {
        if let Some(bind) = self.hints.get(key) {
            bind.shown
        } else {
            false
        }
    }

    /// Updates the state of a keybinding hint. Returns an error if that state is invalid.
    ///
    /// Does nothing if the key is not associated to a keybinding.
    fn set_state(&mut self, key: &str, state: usize) -> Result<()> {
        if let Some(bind) = self.hints.get_mut(key) {
            bind.set_state(state)?;
        }
        Ok(())
    }

    /// Sets all keybinding hints depending on the current state of the application.
    pub fn update(&mut self, state: &State) -> Result<()> {
        // reset keybinding hints to be able to configure them properly for current app state
        self.hints.values_mut().for_each(|x| x.reset());

        // set more specific keybinding hints
        match state {
            State::Default => {
                self.show_and_enable("q");
                self.show_and_enable("s");
                self.show_and_enable("r");
                self.show_and_enable("d");
                self.show_and_enable("i");
                self.show_and_enable("c");
            }
            State::Running(breakpoint_set) => {
                self.show_and_enable("q");
                self.show_and_enable("n");
                self.show_and_enable("d");
                self.show_and_enable("t");
                self.show_and_enable("i");
                self.show_and_enable("c");
                self.show_and_enable("r");
                if *breakpoint_set {
                    self.set_state("r", 1)?;
                }
            }
            State::DebugSelect(_, _) => {
                self.show_and_enable("q");
                self.show_and_enable("d");
                self.show_and_enable("c");
                self.show_and_enable("b");
                self.show_and_enable("j");
                self.show_and_enable(&KeySymbol::ArrowUp.to_string());
                self.show_and_enable(&KeySymbol::ArrowDown.to_string());
                self.set_state("d", 1)?;
            }
            State::Finished(message_shown) => {
                self.show_and_enable("q");
                self.show_and_enable("t");
                if *message_shown {
                    self.show_and_enable("d");
                } else {
                    self.hide("d");
                }
                self.set_state("d", 2)?;
            }
            State::RuntimeError(_, is_playground) => {
                self.show_and_enable("q");

                if *is_playground {
                    self.set_state(&KeySymbol::Enter.to_string(), 2)?;
                    self.show(&KeySymbol::Enter.to_string());
                } else {
                    self.show_and_enable("t");
                }
            }
            State::CustomInstructionError(_, _) | State::BuildProgramError(_) => {
                self.show_and_enable("q");

                self.show_and_enable(&KeySymbol::Enter.to_string());
                self.set_state(&KeySymbol::Enter.to_string(), 2)?;
                self.show(&KeySymbol::Enter.to_string());
            }
            State::CustomInstruction(_, single_instruction) => {
                self.show_and_enable(&KeySymbol::Enter.to_string());
                self.show_and_enable(&KeySymbol::Escape.to_string());
                self.show(&KeySymbol::ArrowUp.to_string());
                self.show_and_enable(&KeySymbol::ArrowDown.to_string());
                self.show_and_enable(&KeySymbol::ArrowLeft.to_string());
                self.show_and_enable(&KeySymbol::ArrowRight.to_string());
                self.show(&KeySymbol::Tab.to_string());
                if single_instruction.input.is_empty() && single_instruction.allowed_values_state.selected().is_none() {
                    self.disable(&KeySymbol::Enter.to_string());
                }
                if single_instruction.allowed_values_state.selected().is_some() {
                    self.enable(&KeySymbol::ArrowUp.to_string());
                }
                if single_instruction.input.is_empty() {
                    self.disable(&KeySymbol::ArrowLeft.to_string());
                    self.disable(&KeySymbol::ArrowRight.to_string());
                }
                if single_instruction.items_to_display().is_empty() {
                    self.disable(&KeySymbol::ArrowDown.to_string());
                }
                if let Some(idx) = single_instruction.allowed_values_state.selected() {
                    if single_instruction.items_to_display().len() == idx + 1 {
                        self.disable(&KeySymbol::ArrowDown.to_string());
                    }
                }
                if single_instruction.allowed_values_state.selected().is_some() {
                    self.set_state(&KeySymbol::Enter.to_string(), 1)?;
                    self.enable(&KeySymbol::Tab.to_string())
                }
            }
            State::Playground(state) => {
                self.show_and_enable(&KeySymbol::Enter.to_string());
                self.show_and_enable(&KeySymbol::Escape.to_string());
                self.show_and_enable(&KeySymbol::ArrowUp.to_string());
                self.show_and_enable(&KeySymbol::ArrowDown.to_string());
                self.show_and_enable(&KeySymbol::ArrowLeft.to_string());
                self.show_and_enable(&KeySymbol::ArrowRight.to_string());
                self.show(&KeySymbol::Tab.to_string());
                self.set_state(&KeySymbol::Escape.to_string(), 1)?;
                if state.input.is_empty() && state.allowed_values_state.selected().is_none() {
                    self.disable(&KeySymbol::Enter.to_string());
                }
                if state.input.is_empty() {
                    self.disable(&KeySymbol::ArrowLeft.to_string());
                    self.disable(&KeySymbol::ArrowRight.to_string());
                }
                if state.items_to_display().is_empty() {
                    self.disable(&KeySymbol::ArrowDown.to_string());
                }
                if let Some(idx) = state.allowed_values_state.selected() {
                    if state.items_to_display().len() == idx + 1 {
                        self.disable(&KeySymbol::ArrowDown.to_string());
                    }
                }
                if state.allowed_values_state.selected().is_some() {
                    self.set_state(&KeySymbol::Enter.to_string(), 1)?;
                    self.enable(&KeySymbol::Tab.to_string());
                } else {
                    self.disable(&KeySymbol::ArrowUp.to_string());
                }
            }
        }
        Ok(())
    }
}

/// Returns the default keybindings.
fn default_keybindings() -> Result<HashMap<String, KeybindingHint>> {
    let mut hints = HashMap::new();
    hints.insert(
        "q".to_string(),
        KeybindingHint::new(0, &format!("q|{}", KeySymbol::Escape), "Quit"),
    );
    hints.insert("s".to_string(), KeybindingHint::new(2, "s", "Start"));
    hints.insert(
        "n".to_string(),
        KeybindingHint::new_many(vec![4], "n", vec!["Run next instruction"])?,
    );
    hints.insert(
        "r".to_string(),
        KeybindingHint::new_many(
            vec![2, 2],
            "r",
            vec!["Run to end", "Run to next breakpoint"],
        )?,
    );
    hints.insert(
        "d".to_string(),
        KeybindingHint::new_many(
            vec![7, 7, 7],
            "d",
            vec![
                "Enter debug select mode",
                "Exit debug select mode",
                "Dismiss message",
            ],
        )?,
    );
    hints.insert("t".to_string(), KeybindingHint::new(1, "t", "Reset"));
    hints.insert(
        "b".to_string(),
        KeybindingHint::new(8, "b", "Toggle breakpoint"),
    );
    hints.insert(
        "j".to_string(),
        KeybindingHint::new(11, "j", "Jump to line"),
    );
    hints.insert(
        KeySymbol::ArrowUp.to_string(),
        KeybindingHint::new(12, &KeySymbol::ArrowUp.to_string(), "Up"),
    );
    hints.insert(
        KeySymbol::ArrowDown.to_string(),
        KeybindingHint::new(13, &KeySymbol::ArrowDown.to_string(), "Down"),
    );
    hints.insert(
        "i".to_string(),
        KeybindingHint::new(9, "i", "Run custom instruction"),
    );
    hints.insert(
        "c".to_string(),
        KeybindingHint::new(10, "c", "Toggle call stack"),
    );
    hints.insert(
        KeySymbol::ArrowLeft.to_string(),
        KeybindingHint::new(10, &KeySymbol::ArrowLeft.to_string(), "Cursor left"),
    );
    hints.insert(
        KeySymbol::ArrowRight.to_string(),
        KeybindingHint::new(11, &KeySymbol::ArrowRight.to_string(), "Cursor right"),
    );
    hints.insert(
        KeySymbol::Enter.to_string(),
        KeybindingHint::new_many(
            vec![5, 5, 5],
            &KeySymbol::Enter.to_string(),
            vec![
                "Run entered instruction",
                "Run selected instruction",
                "Close",
            ],
        )?,
    );
    hints.insert(
        KeySymbol::Escape.to_string(),
        KeybindingHint::new_many(
            vec![1, 1],
            &KeySymbol::Escape.to_string(),
            vec!["Cancel", "Exit"],
        )?,
    );
    hints.insert(
        KeySymbol::Tab.to_string(),
        KeybindingHint::new(9, &KeySymbol::Tab.to_string(), "Fill in selected"),
    );
    Ok(hints)
}

/// Used organize hints of keybinds
#[derive(Clone, PartialEq, Debug)]
pub struct KeybindingHint {
    /// Order is used to specify the position at which it should be displayed in the list.
    ///
    /// Smaller values are displayed before higher values.
    orders: Vec<usize>,
    pub key: String,
    /// All labels that are associated to this keybinding.
    labels: Vec<String>,
    /// If true the keybinding hint should be displayed as enabled.
    /// If false the keybinding hint should be displayed as disabled.
    enabled: bool,
    /// If true the keybinding hint should be displayed in the ui.
    shown: bool,
    /// Stores the index of the label and order that is currently active.
    state: usize,
}

impl KeybindingHint {
    /// Construct a new keybinding hint with a single possible state.
    ///
    /// `enabled` and `shown` are initially set to false.
    fn new(order: usize, key: &str, label: &str) -> Self {
        Self {
            orders: vec![order],
            key: key.to_string(),
            labels: vec![label.to_string()],
            enabled: false,
            shown: false,
            state: 0,
        }
    }

    /// Construct a new keybinding hint with a single possible state.
    ///
    /// `enabled` and `shown` are initially set to false.
    #[cfg(test)]
    fn new_with_state(order: usize, key: &str, label: &str, enabled: bool, shown: bool) -> Self {
        Self {
            orders: vec![order],
            key: key.to_string(),
            labels: vec![label.to_string()],
            enabled,
            shown,
            state: 0,
        }
    }

    /// Construct a new keybinding hint that can be used with multiple labels and orders.
    ///
    /// Orders and labels are required to have the same length, otherwise this function will return an error.
    ///
    /// `enabled` and `shown` are initially set to false.
    fn new_many(orders: Vec<usize>, key: &str, labels: Vec<&str>) -> Result<Self> {
        if orders.len() != labels.len() {
            return Err(anyhow!("Length of orders and labels is not equal!"));
        }
        Ok(Self {
            orders,
            key: key.to_string(),
            labels: labels.iter().map(|f| f.to_string()).collect(),
            enabled: false,
            shown: false,
            state: 0,
        })
    }

    /// Reset the keybinding hint, meaning that the fields enabled and shown are set to false.
    fn reset(&mut self) {
        self.enabled = false;
        self.shown = false;
    }

    /// Updates the state of this keybinding.
    ///
    /// Returns an error when the state is invalid. The state is invalid if it is to large to index orders and labels.
    fn set_state(&mut self, state: usize) -> Result<()> {
        if state > self.orders.len() {
            Err(anyhow!(
                "State invalid! Should be between 0 and {}. Was {}.",
                self.orders.len() - 1,
                state
            ))
        } else {
            self.state = state;
            Ok(())
        }
    }

    fn order(&self) -> usize {
        self.orders[self.state]
    }

    fn label(&self) -> String {
        self.labels[self.state].clone()
    }
}

pub enum KeySymbol {
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Enter,
    Escape,
    Tab,
}

impl Display for KeySymbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeySymbol::ArrowUp => write!(f, "\u{2191}"),
            KeySymbol::ArrowDown => write!(f, "\u{2193}"),
            KeySymbol::ArrowLeft => write!(f, "\u{2190}"),
            KeySymbol::ArrowRight => write!(f, "\u{2192}"),
            KeySymbol::Enter => write!(f, "\u{23ce}"),
            KeySymbol::Escape => write!(f, "\u{238b}"),
            KeySymbol::Tab => write!(f, "\u{21e5}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::app::ui::style::{SharedTheme, Theme};

    use super::{KeybindingHint, KeybindingHints};

    fn test_keybinding_hints() -> KeybindingHints {
        let mut hints = HashMap::new();
        hints.insert("a".to_string(), KeybindingHint::new(0, "a", "test_label_1"));
        hints.insert("b".to_string(), KeybindingHint::new(0, "b", "test_label_2"));
        hints.insert("c".to_string(), KeybindingHint::new(0, "c", "test_label_3"));
        hints.insert("d".to_string(), KeybindingHint::new(0, "d", "test_label_4"));
        let mut hints = KeybindingHints {
            hints,
            theme: SharedTheme::new(Theme::default()),
        };
        hints.show_and_enable("a");
        hints.enable("c");
        hints.show("d");
        hints
    }

    #[test]
    fn test_keybinding_hints_active_keybinds() {
        let hints = test_keybinding_hints();

        assert!(hints
            .active_keybinds()
            .contains(&KeybindingHint::new_with_state(
                0,
                "a",
                "test_label_1",
                true,
                true
            )));
        assert!(hints
            .active_keybinds()
            .contains(&KeybindingHint::new_with_state(
                0,
                "d",
                "test_label_4",
                false,
                true
            )));
    }

    #[test]
    fn test_keybinding_hints_enable() {
        let mut hints = test_keybinding_hints();
        hints.enable("b");
        assert!(hints.hints.get("b").unwrap().enabled);
    }

    #[test]
    fn test_keybinding_hints_disable() {
        let mut hints = test_keybinding_hints();
        hints.disable("a");
        assert!(!hints
            .active_keybinds()
            .contains(&KeybindingHint::new(0, "a", "test_label_1",)))
    }

    #[test]
    fn test_keybinding_hints_status() {
        let hints = test_keybinding_hints();
        assert!(hints._status("a"));
        assert!(!hints._status("b"));
        assert!(!hints._status("e"));
    }

    #[test]
    fn test_keybinding_hints_set_state() {
        let mut hints = test_keybinding_hints();
        hints.hints.insert(
            "c".to_string(),
            KeybindingHint::new_many(vec![0, 1], "c", vec!["State1", "State2"]).unwrap(),
        );
        assert_eq!(hints.hints.get("c").unwrap().label(), "State1");
        hints.set_state("c", 1).unwrap();
        assert_eq!(hints.hints.get("c").unwrap().label(), "State2");
    }

    #[test]
    fn test_keybinding_hint_new_many_err() {
        let res = KeybindingHint::new_many(vec![0, 2], "a", vec![]);
        assert!(res.is_err());
    }

    #[test]
    fn test_keybinding_hint_order() {
        let hint = KeybindingHint::new(0, "a", "test1");
        assert_eq!(hint.order(), 0);
        let mut hint = KeybindingHint::new_many(vec![0, 1], "a", vec!["test1a", "test1b"]).unwrap();
        assert_eq!(hint.order(), 0);
        hint.set_state(1).unwrap();
        assert_eq!(hint.order(), 1);
    }

    #[test]
    fn test_keybinding_hint_set_state() {
        let mut hint = KeybindingHint::new_many(vec![0, 1], "a", vec!["test1a", "test1b"]).unwrap();
        assert_eq!(hint.state, 0);
        hint.set_state(1).unwrap();
        assert_eq!(hint.state, 1);
        assert!(hint.set_state(3).is_err());
    }

    #[test]
    fn test_keybinding_hint_show() {
        let mut hints = test_keybinding_hints();
        assert!(!hints
            .active_keybinds()
            .contains(&KeybindingHint::new(0, "c", "test_label_3",)));
        hints.show("c");
        assert!(hints
            .active_keybinds()
            .contains(&KeybindingHint::new_with_state(
                0,
                "c",
                "test_label_3",
                true,
                true
            )));
    }

    #[test]
    fn test_keybinding_hint_hide() {
        let mut hints = test_keybinding_hints();
        assert!(hints
            .active_keybinds()
            .contains(&KeybindingHint::new_with_state(
                0,
                "d",
                "test_label_4",
                false,
                true
            )));
        hints.hide("d");
        assert!(!hints
            .active_keybinds()
            .contains(&KeybindingHint::new_with_state(
                0,
                "d",
                "test_label_4",
                false,
                true
            )));
    }
}
