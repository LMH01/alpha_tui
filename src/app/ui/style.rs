use std::rc::Rc;

use clap::ValueEnum;
use ratatui::style::{Color, Style};
use serde::{Deserialize, Serialize};

pub type SharedTheme = Rc<Theme>;
pub type SharedSyntaxHighlightingTheme = Rc<SyntaxHighlightingTheme>;

/// The theme of the application.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Theme {
    sh_theme: SyntaxHighlightingTheme,
    background: Color,
    foreground: Color,
    breakpoint_accent: Color,
    error: Color,
    code_area_default: Color,
    list_item_highlight_fg: Color,
    list_item_highlight_bg: Color,
    line_numbers: Color,
    execution_finished_popup_border: Color,
    keybindings_fg: Color,
    keybindings_disabled_fg: Color,
    keybindings_bg: Color,
    custom_instruction_accent_fg: Color,
    memory_block_border: Color,
    internal_memory_block_border: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Theme::dracula()
    }
}

impl From<&BuildInTheme> for Theme {
    fn from(value: &BuildInTheme) -> Self {
        match value {
            BuildInTheme::DefaultOld => Theme::default_old(),
            BuildInTheme::Dracula => Theme::dracula(),
            BuildInTheme::Gray => serde_json::from_str(r#"{"sh_theme":{"assignment":"White","op":"White","cmp":"White","label":"White","build_in":"White","accumulator":"White","gamma":"White","memory_cell_outer":"White","memory_cell_inner":"White","index_memory_cell_outer":"White","index_memory_cell_index_outer":"White","constant":"White","comment":"White"},"background":"Black","foreground":"White","breakpoint_accent":"DarkGray","error":"White","code_area_default":"White","list_item_highlight_fg":"White","list_item_highlight_bg":"DarkGray","line_numbers":"White","execution_finished_popup_border":"White","keybindings_fg":"White","keybindings_disabled_fg":"DarkGray","keybindings_bg":"DarkGray","custom_instruction_accent_fg":"White","memory_block_border":"White","internal_memory_block_border":"White"}"#).unwrap(),
        }
    }
}

impl Theme {
    /// The default theme of the app before `v1.5.0``.
    pub fn default_old() -> Self {
        Self {
            sh_theme: SyntaxHighlightingTheme::default(),
            background: Color::default(),
            foreground: Color::White,
            breakpoint_accent: Color::Magenta,
            error: Color::Red,
            code_area_default: Color::Green,
            list_item_highlight_fg: Color::White,
            list_item_highlight_bg: Color::Rgb(98, 114, 164),
            line_numbers: Color::White,
            execution_finished_popup_border: Color::Green,
            keybindings_fg: Color::White,
            keybindings_disabled_fg: Color::DarkGray,
            keybindings_bg: Color::Rgb(98, 114, 164),
            custom_instruction_accent_fg: Color::Cyan,
            memory_block_border: Color::LightBlue,
            internal_memory_block_border: Color::Yellow,
        }
    }

    /// The dracula theme.
    pub fn dracula() -> Self {
        Self {
            sh_theme: SyntaxHighlightingTheme::default(),
            background: BACKGROUND,
            foreground: FOREGROUND,
            breakpoint_accent: PURPLE,
            error: RED,
            code_area_default: GREEN,
            list_item_highlight_fg: FOREGROUND,
            list_item_highlight_bg: SELECTION,
            line_numbers: FOREGROUND,
            execution_finished_popup_border: GREEN,
            keybindings_fg: FOREGROUND,
            keybindings_disabled_fg: COMMENT,
            keybindings_bg: COMMENT,
            custom_instruction_accent_fg: CYAN,
            memory_block_border: YELLOW,
            internal_memory_block_border: ORANGE,
        }
    }

    pub fn syntax_highlighting_theme(&self) -> SharedSyntaxHighlightingTheme {
        Rc::new(self.sh_theme.clone())
    }

    pub fn custom_instruction(&self) -> Style {
        Style::default().fg(self.custom_instruction_accent_fg)
    }

    pub fn list_item_highlight(&self, breakpoint_mode: bool) -> Style {
        let style = Style::default();
        if breakpoint_mode {
            style
                .bg(self.breakpoint_accent)
                .fg(self.list_item_highlight_fg)
        } else {
            style
                .bg(self.list_item_highlight_bg)
                .fg(self.list_item_highlight_fg)
        }
    }

    pub fn keybinding_hints(&self, enabled: bool) -> Style {
        let style = Style::default();
        if enabled {
            style.fg(self.keybindings_fg).bg(self.keybindings_bg)
        } else {
            style.fg(self.keybindings_disabled_fg).bg(self.background)
        }
    }

    pub fn error_block(&self) -> Style {
        Style::default().bg(self.background).fg(self.foreground)
    }

    pub fn error_block_border(&self) -> Style {
        Style::default().fg(self.error)
    }

    pub fn code_block(&self) -> Style {
        Style::default().bg(self.background)
    }

    pub fn code_block_border(&self) -> Style {
        Style::default().fg(self.code_area_default)
    }

    pub fn breakpoint_border(&self) -> Style {
        Style::default().fg(self.breakpoint_accent)
    }

    pub fn breakpoint(&self) -> Style {
        Style::default().fg(self.breakpoint_accent)
    }

    pub fn breakpoint_block(&self) -> Style {
        Style::default()
            .bg(self.background)
            .fg(self.breakpoint_accent)
    }

    pub fn memory_block(&self) -> Style {
        Style::default()
            .fg(self.memory_block_border)
            .bg(self.background)
    }

    pub fn memory_block_border(&self) -> Style {
        Style::default().fg(self.memory_block_border)
    }

    pub fn internal_memory_block(&self) -> Style {
        Style::default()
            .fg(self.internal_memory_block_border)
            .bg(self.background)
    }

    pub fn internal_memory_block_border(&self) -> Style {
        Style::default().fg(self.internal_memory_block_border)
    }

    pub fn execution_finished_popup_border(&self) -> Style {
        Style::default().fg(self.execution_finished_popup_border)
    }

    pub fn keybinding_hint_paragraph(&self) -> Style {
        Style::default().bg(self.background)
    }

    pub fn single_instruction_block(&self) -> Style {
        Style::default().bg(self.background).fg(self.foreground)
    }

    pub fn execution_finished_block(&self) -> Style {
        Style::default().bg(self.background).fg(self.foreground)
    }

    pub fn line_numbers(&self) -> Style {
        Style::default().bg(self.background).fg(self.line_numbers)
    }

    // code syntax highlighting styles start here
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct SyntaxHighlightingTheme {
    assignment: Color,
    op: Color,
    cmp: Color,
    label: Color,
    build_in: Color,
    accumulator: Color,
    gamma: Color,
    /// `p( )` part of the memory cell.
    memory_cell_outer: Color,
    /// The part in the center of the brackets.
    memory_cell_inner: Color,
    /// `p( )` part of the index memory cell.
    index_memory_cell_outer: Color,
    index_memory_cell_index_outer: Color,
    constant: Color,
    comment: Color,
}

impl Default for SyntaxHighlightingTheme {
    fn default() -> Self {
        Self {
            assignment: PINK,
            op: PINK,
            cmp: PINK,
            label: GREEN,
            build_in: PINK,
            accumulator: FOREGROUND,
            gamma: PURPLE,
            memory_cell_outer: CYAN,
            memory_cell_inner: FOREGROUND,
            index_memory_cell_outer: CYAN,
            index_memory_cell_index_outer: GREEN,
            constant: PURPLE,
            comment: COMMENT,
        }
    }
}

impl SyntaxHighlightingTheme {
    pub fn assignment(&self) -> Style {
        Style::default().fg(self.assignment)
    }

    pub fn op(&self) -> Style {
        Style::default().fg(self.op)
    }

    pub fn cmp(&self) -> Style {
        Style::default().fg(self.cmp)
    }

    pub fn label(&self, enable_syntax_highlighting: bool) -> Style {
        let style = Style::default();
        if enable_syntax_highlighting {
            style.fg(self.label)
        } else {
            style
        }
    }

    pub fn build_in(&self, enable_syntax_highlighting: bool) -> Style {
        let style = Style::default();
        if enable_syntax_highlighting {
            style.fg(self.build_in)
        } else {
            style
        }
    }

    pub fn accumulator(&self) -> Style {
        Style::default().fg(self.accumulator)
    }

    pub fn gamma(&self) -> Style {
        Style::default().fg(self.gamma)
    }

    pub fn memory_cell_outer(&self) -> Style {
        Style::default().fg(self.memory_cell_outer)
    }

    pub fn memory_cell_inner(&self) -> Style {
        Style::default().fg(self.memory_cell_inner)
    }

    pub fn index_memory_cell_outer(&self) -> Style {
        Style::default().fg(self.index_memory_cell_outer)
    }

    pub fn index_memory_cell_index_outer(&self) -> Style {
        Style::default().fg(self.index_memory_cell_index_outer)
    }

    pub fn constant(&self) -> Style {
        Style::default().fg(self.constant)
    }

    pub fn comment(&self, enable_syntax_highlighting: bool) -> Style {
        let style = Style::default();
        if enable_syntax_highlighting {
            style.fg(self.comment)
        } else {
            style
        }
    }
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

#[derive(Serialize, Deserialize, Clone, Debug, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum BuildInTheme {
    Dracula,
    DefaultOld,
    Gray,
}
