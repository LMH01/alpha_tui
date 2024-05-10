use std::rc::Rc;

use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize};

pub type SharedTheme = Rc<Theme>;
pub type SharedSyntaxHighlightingTheme = Rc<SyntaxHighlightingTheme>;

/// The theme of the application.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Theme {
    sh_theme: SyntaxHighlightingTheme,
    breakpoint_accent: Color,
    error: Color,
    code_area_default: Color,
    list_item_highlight: Color,
    execution_finished_popup_border: Color,
    keybindings_fg: Color,
    keybindings_disabled_fg: Color,
    keybindings_bg: Color,
    keybindings_disabled_bg: Color,
    custom_instruction_accent_fg: Color,
    memory_block_border: Color,
    internal_memory_block_border: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Theme::dracula()
    }
}

impl Theme {
    /// The default theme of the app before `v1.5.0``.
    pub fn default_old() -> Self {
        Self {
            sh_theme: SyntaxHighlightingTheme::default(),
            breakpoint_accent: Color::Magenta,
            error: Color::Red,
            code_area_default: Color::Green,
            list_item_highlight: Color::Rgb(98, 114, 164),
            execution_finished_popup_border: Color::Green,
            keybindings_fg: Color::White,
            keybindings_disabled_fg: Color::DarkGray,
            keybindings_bg: Color::Rgb(98, 114, 164),
            keybindings_disabled_bg: Color::Black,
            custom_instruction_accent_fg: Color::Cyan,
            memory_block_border: Color::LightBlue,
            internal_memory_block_border: Color::Yellow,
        }
    }

    /// The dracula theme.
    pub fn dracula() -> Self {
        todo!()
    }

    pub fn syntax_highlighting_theme(&self) -> SharedSyntaxHighlightingTheme {
        Rc::new(self.sh_theme.clone())
    }

    pub fn custom_instruction(&self) -> Style {
        Style::default().fg(self.custom_instruction_accent_fg)
    }

    pub fn list_item_highlight(&self, breakpoint_mode: bool) -> Style {
        let style = Style::default().add_modifier(Modifier::BOLD);
        if breakpoint_mode {
            style.fg(self.breakpoint_accent)
        } else {
            style.fg(self.list_item_highlight)
        }
    }

    pub fn keybinding_hints(&self, enabled: bool) -> Style {
        let style = Style::default();
        if enabled {
            style.fg(self.keybindings_fg).bg(self.keybindings_bg)
        } else {
            style
                .fg(self.keybindings_disabled_fg)
                .bg(self.keybindings_disabled_bg)
        }
    }

    pub fn error_border(&self) -> Style {
        Style::default().fg(self.error)
    }

    pub fn breakpoint_border(&self) -> Style {
        Style::default().fg(self.breakpoint_accent)
    }

    pub fn breakpoint(&self) -> Style {
        Style::default().fg(self.breakpoint_accent)
    }

    pub fn code_area_border(&self) -> Style {
        Style::default().fg(self.code_area_default)
    }

    pub fn memory_block_border(&self) -> Style {
        Style::default().fg(self.memory_block_border)
    }

    pub fn internal_memory_block_border(&self) -> Style {
        Style::default().fg(self.internal_memory_block_border)
    }

    pub fn execution_finished_popup_border(&self) -> Style {
        Style::default().fg(self.execution_finished_popup_border)
    }

    // code syntax highlighting styles start here
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct SyntaxHighlightingTheme {
    assignment: Color,
    op: Color,
    label: Color,
    build_in: Color,
}

impl Default for SyntaxHighlightingTheme {
    fn default() -> Self {
        Self {
            assignment: PINK,
            op: Default::default(),
            label: Default::default(),
            build_in: Default::default(),
        }
    }
}

impl SyntaxHighlightingTheme {
    pub fn assignment_span(&self) -> Style {
        Style::default().fg(self.assignment)
    }

    //pub fn op_span(&self) -> Style {
    //
    //}
}

// dracula theme color palette
// is used for the default application theme
const BACKGROUND: Color = Color::Rgb(40, 42, 54);
const FOREGROUND: Color = Color::Rgb(248, 248, 242);
const SELECTION: Color = Color::Rgb(68, 71, 90);
const COMMENT: Color = Color::Rgb(98, 114, 164);
const RED: Color = Color::Rgb(255, 85, 85);
const ORANGE: Color = Color::Rgb(255, 184, 108);
const YELLOW: Color = Color::Rgb(241, 250, 140);
const GREEN: Color = Color::Rgb(80, 250, 123);
const PURPLE: Color = Color::Rgb(189, 147, 249);
const CYAN: Color = Color::Rgb(139, 233, 253);
const PINK: Color = Color::Rgb(255, 121, 198);
