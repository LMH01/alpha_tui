use std::collections::HashMap;

use anyhow::{anyhow, Result};
use ratatui::{
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
};

use super::State;

/// Manages all keybinding hints.
pub struct KeybindingHints {
    hints: HashMap<String, KeybindingHint>,
}

impl KeybindingHints {
    pub fn new() -> Result<Self> {
        Ok(Self {
            hints: default_keybindings()?,
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
                styled_keybinds_row.push(Span::from(" ").bg(Color::default()));
                line_length += 1;
            }
            let text = format!("{} [{}]", hint.label(), hint.key);
            if line_length > width as usize - text.len() {
                line_length = 0;
                styled_keybinds.push(Line::from(styled_keybinds_row));
                styled_keybinds_row = Vec::new();
            }
            line_length += text.len();
            let style = if hint.enabled {
                Style::default()
                    .fg(super::KEYBINDS_FG)
                    .bg(super::KEYBINDS_BG)
            } else {
                Style::default()
                    .fg(super::KEYBINDS_DISABLED_FG)
                    .bg(super::KEYBINDS_DISABLED_BG)
            };
            styled_keybinds_row.push(Span::from(text).style(style));
        }
        styled_keybinds.push(Line::from(styled_keybinds_row));

        let hight = styled_keybinds.len() as u16;
        let keybinding_hints = Paragraph::new(styled_keybinds);
        (keybinding_hints, hight)
    }

    /// Returns a list of keybinding hints that are currently active.
    pub fn active_keybinds(&self) -> Vec<KeybindingHint> {
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
    pub fn _enable(&mut self, key: &str) {
        if let Some(bind) = self.hints.get_mut(key) {
            bind.enabled = true;
        }
    }

    /// Use to disable a keybinding hint.
    ///
    /// Disabled keybinding hints are still displayed but grayed out.
    ///
    /// Does nothing if the key is not associated to a keybinding.
    pub fn disable(&mut self, key: &str) {
        if let Some(bind) = self.hints.get_mut(key) {
            bind.enabled = false;
        }
    }

    /// Use to show a keybinding hint.
    ///
    /// Does nothing if the key is not associated to a binding.
    pub fn show(&mut self, key: &str) {
        if let Some(bind) = self.hints.get_mut(key) {
            bind.shown = true;
        }
    }

    /// Use to hide a keybinding hint.
    ///
    /// Does nothing if the key is not associated to a binding.
    pub fn hide(&mut self, key: &str) {
        if let Some(bind) = self.hints.get_mut(key) {
            bind.shown = false;
        }
    }

    /// Checks the status of the keybinding hint.
    ///
    /// Returns `true` if the keybinding hint is shown or `false` if the keybinding hint is hidden or was not found.
    pub fn _status(&self, key: &str) -> bool {
        if let Some(bind) = self.hints.get(key) {
            bind.shown
        } else {
            false
        }
    }

    /// Updates the state of a keybinding hint. Returns an error if that state is invalid.
    ///
    /// Does nothing if the key is not associated to a keybinding.
    pub fn set_state(&mut self, key: &str, state: usize) -> Result<()> {
        if let Some(bind) = self.hints.get_mut(key) {
            bind.set_state(state)?;
        }
        Ok(())
    }

    /// Sets all keybinding hints depending on the current state of the application.
    pub fn update(&mut self, state: &State) -> Result<()> {
        // reset keybindings back to default (I know that this is not that optimal for the performance)
        self.hints =
            default_keybindings().expect("Keybinding hints should be properly initialized");

        // set more specific keybinding hints
        match state {
            State::Running(breakpoint_set) => {
                self.hide("s");
                self.show("r");
                self.show("t");
                self.show("n");
                self.show("i");
                if *breakpoint_set {
                    self.set_state("r", 1)?;
                }
            }
            State::DebugSelect(_, _) => {
                self.hide("s");
                self.show("b");
                self.show("j");
                self.show(&KeySymbol::ArrowUp.to_string());
                self.show(&KeySymbol::ArrowDown.to_string());
                self.set_state("d", 1)?;
            }
            State::Finished(_) => {
                self.hide("s");
                self.hide("c");
                self.show("t");
                self.set_state("d", 2)?;
            }
            State::RuntimeError(_, _) => {
                self.hide("c");
                self.hide("s");
                self.hide("d");
                self.show("t");
            }
            State::CustomInstruction(state) => {
                self.hide("c");
                self.hide("s");
                self.hide("d");
                self.hide("q");
                self.show(&KeySymbol::Enter.to_string());
                self.show(&KeySymbol::Escape.to_string());
                self.show(&KeySymbol::ArrowUp.to_string());
                self.show(&KeySymbol::ArrowDown.to_string());
                self.show(&KeySymbol::ArrowLeft.to_string());
                self.show(&KeySymbol::ArrowRight.to_string());
                if state.input.is_empty() && state.allowed_values_state.selected().is_none() {
                    self.disable(&KeySymbol::Enter.to_string());
                }
                if state.allowed_values_state.selected().is_some() {
                    self.set_state(&KeySymbol::Enter.to_string(), 1)?;
                }
            }
            State::Sandbox(state) => {
                self.hide("c");
                self.hide("s");
                self.hide("d");
                self.hide("q");
                self.show(&KeySymbol::Enter.to_string());
                self.show(&KeySymbol::Escape.to_string());
                self.show(&KeySymbol::ArrowUp.to_string());
                self.show(&KeySymbol::ArrowDown.to_string());
                self.show(&KeySymbol::ArrowLeft.to_string());
                self.show(&KeySymbol::ArrowRight.to_string());
                self.set_state(&KeySymbol::Escape.to_string(), 1)?;
                if state.input.is_empty() && state.allowed_values_state.selected().is_none() {
                    self.disable(&KeySymbol::Enter.to_string());
                }
                if state.allowed_values_state.selected().is_some() {
                    self.set_state(&KeySymbol::Enter.to_string(), 1)?;
                }
            }
            _ => (),
        }
        Ok(())
    }
}

/// Returns the default keybindings.
fn default_keybindings() -> Result<HashMap<String, KeybindingHint>> {
    let mut hints = HashMap::new();
    hints.insert(
        "q".to_string(),
        KeybindingHint::new(0, "q", "Quit", true, true),
    );
    hints.insert(
        "s".to_string(),
        KeybindingHint::new(4, "s", "Start", true, true),
    );
    hints.insert(
        "n".to_string(),
        KeybindingHint::new_many(vec![4], "n", vec!["Run next instruction"], true, false)?,
    );
    hints.insert(
        "r".to_string(),
        KeybindingHint::new_many(
            vec![2, 2],
            "r",
            vec!["Run to end", "Run to next breakpoint"],
            true,
            false,
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
            true,
            true,
        )?,
    );
    hints.insert(
        "t".to_string(),
        KeybindingHint::new(1, "t", "Reset", true, false),
    );
    hints.insert(
        "b".to_string(),
        KeybindingHint::new(8, "b", "Toggle breakpoint", true, false),
    );
    hints.insert(
        "j".to_string(),
        KeybindingHint::new(11, "j", "Jump to line", true, false),
    );
    hints.insert(
        KeySymbol::ArrowUp.to_string(),
        KeybindingHint::new(12, &KeySymbol::ArrowUp.to_string(), "Up", true, false),
    );
    hints.insert(
        KeySymbol::ArrowDown.to_string(),
        KeybindingHint::new(13, &KeySymbol::ArrowDown.to_string(), "Down", true, false),
    );
    hints.insert(
        "i".to_string(),
        KeybindingHint::new(9, "i", "Run custom instruction", true, false),
    );
    hints.insert(
        "c".to_string(),
        KeybindingHint::new(10, "c", "Toggle call stack", true, true),
    );
    hints.insert(
        KeySymbol::ArrowLeft.to_string(),
        KeybindingHint::new(
            10,
            &KeySymbol::ArrowLeft.to_string(),
            "Cursor left",
            true,
            false,
        ),
    );
    hints.insert(
        KeySymbol::ArrowRight.to_string(),
        KeybindingHint::new(
            11,
            &KeySymbol::ArrowRight.to_string(),
            "Cursor right",
            true,
            false,
        ),
    );
    hints.insert(
        KeySymbol::Enter.to_string(),
        KeybindingHint::new_many(
            vec![5, 5],
            &KeySymbol::Enter.to_string(),
            vec!["Run entered instruction", "Run selected instruction"],
            true,
            false,
        )?,
    );
    hints.insert(
        KeySymbol::Escape.to_string(),
        KeybindingHint::new_many(vec![1, 1], &KeySymbol::Escape.to_string(), vec!["Cancel", "Exit"], true, false)?,
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
    pub enabled: bool,
    /// If true the keybinding hint should be displayed in the ui.
    pub shown: bool,
    /// Stores the index of the label and order that is currently active.
    state: usize,
}

impl KeybindingHint {
    /// Construct a new keybinding hint with a single possible state.
    fn new(order: usize, key: &str, label: &str, enabled: bool, shown: bool) -> Self {
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
    fn new_many(
        orders: Vec<usize>,
        key: &str,
        labels: Vec<&str>,
        enabled: bool,
        shown: bool,
    ) -> Result<Self> {
        if orders.len() != labels.len() {
            return Err(anyhow!("Length of orders and labels is not equal!"));
        }
        Ok(Self {
            orders,
            key: key.to_string(),
            labels: labels.iter().map(|f| f.to_string()).collect(),
            enabled,
            shown,
            state: 0,
        })
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
}

impl ToString for KeySymbol {
    fn to_string(&self) -> String {
        match self {
            KeySymbol::ArrowUp => "\u{2191}".to_string(),
            KeySymbol::ArrowDown => "\u{2193}".to_string(),
            KeySymbol::ArrowLeft => "\u{2190}".to_string(),
            KeySymbol::ArrowRight => "\u{2192}".to_string(),
            KeySymbol::Enter => "\u{23ce}".to_string(),
            KeySymbol::Escape => "\u{238b}".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{KeybindingHint, KeybindingHints};

    fn test_keybinding_hints() -> KeybindingHints {
        let mut hints = HashMap::new();
        hints.insert(
            "a".to_string(),
            KeybindingHint::new(0, "a", "test_label_1", true, true),
        );
        hints.insert(
            "b".to_string(),
            KeybindingHint::new(0, "b", "test_label_2", false, false),
        );
        hints.insert(
            "c".to_string(),
            KeybindingHint::new(0, "c", "test_label_3", true, false),
        );
        hints.insert(
            "d".to_string(),
            KeybindingHint::new(0, "d", "test_label_4", false, true),
        );
        let hints = KeybindingHints { hints };
        hints
    }

    #[test]
    fn test_keybinding_hints_active_keybinds() {
        let hints = test_keybinding_hints();

        assert!(hints.active_keybinds().contains(&KeybindingHint::new(
            0,
            "a",
            "test_label_1",
            true,
            true
        )));
        assert!(hints.active_keybinds().contains(&KeybindingHint::new(
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
        hints._enable("b");
        assert!(hints.hints.get("b").unwrap().enabled);
    }

    #[test]
    fn test_keybinding_hints_disable() {
        let mut hints = test_keybinding_hints();
        hints.disable("a");
        assert!(!hints.active_keybinds().contains(&KeybindingHint::new(
            0,
            "a",
            "test_label_1",
            true,
            true
        )))
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
            KeybindingHint::new_many(vec![0, 1], "c", vec!["State1", "State2"], true, true)
                .unwrap(),
        );
        assert_eq!(hints.hints.get("c").unwrap().label(), "State1");
        hints.set_state("c", 1).unwrap();
        assert_eq!(hints.hints.get("c").unwrap().label(), "State2");
    }

    #[test]
    fn test_keybinding_hint_new_many_err() {
        let res = KeybindingHint::new_many(vec![0, 2], "a", vec![], true, true);
        assert!(res.is_err());
    }

    #[test]
    fn test_keybinding_hint_order() {
        let hint = KeybindingHint::new(0, "a", "test1", true, true);
        assert_eq!(hint.order(), 0);
        let mut hint =
            KeybindingHint::new_many(vec![0, 1], "a", vec!["test1a", "test1b"], true, true)
                .unwrap();
        assert_eq!(hint.order(), 0);
        hint.set_state(1).unwrap();
        assert_eq!(hint.order(), 1);
    }

    #[test]
    fn test_keybinding_hint_set_state() {
        let mut hint =
            KeybindingHint::new_many(vec![0, 1], "a", vec!["test1a", "test1b"], true, true)
                .unwrap();
        assert_eq!(hint.state, 0);
        hint.set_state(1).unwrap();
        assert_eq!(hint.state, 1);
        assert!(hint.set_state(3).is_err());
    }

    #[test]
    fn test_keybinding_hint_show() {
        let mut hints = test_keybinding_hints();
        assert!(!hints.active_keybinds().contains(&KeybindingHint::new(
            0,
            "c",
            "test_label_3",
            true,
            false
        )));
        hints.show("c");
        assert!(hints.active_keybinds().contains(&KeybindingHint::new(
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
        assert!(hints.active_keybinds().contains(&KeybindingHint::new(
            0,
            "d",
            "test_label_4",
            false,
            true
        )));
        hints.hide("d");
        assert!(!hints.active_keybinds().contains(&KeybindingHint::new(
            0,
            "d",
            "test_label_4",
            false,
            true
        )));
    }
}
