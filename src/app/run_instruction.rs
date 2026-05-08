use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::Style,
    widgets::{Block, Borders, Clear, List, ListState, Paragraph},
};
use trie_rs::TrieBuilder;

use super::ui::style::SharedTheme;

#[derive(Debug, PartialEq, Clone)]
pub struct SingleInstruction {
    // Input currently entered by the user
    pub input: String,
    pub cursor_position: usize,
    /// Stores the state of the list that stores selectable items.
    ///
    /// If None, currently no item is selected, if some an item is selected.
    pub allowed_values_state: ListState,
    /// List of instructions that where already entered manually.
    ///
    /// Used to populate the list.
    pub executed_instructions: Vec<String>,
    theme: SharedTheme,
}

impl SingleInstruction {
    /// Create a new single instruction.
    pub fn new(executed_instructions: &[String], theme: &SharedTheme) -> Self {
        Self {
            input: String::new(),
            cursor_position: 0,
            allowed_values_state: ListState::default(),
            executed_instructions: executed_instructions.to_owned(),
            theme: theme.clone(),
        }
    }

    /// Draws this single instruction.
    ///
    /// If `floating` is set, a centered area is created inside `r`, where this window is drawn.
    /// If it is false, the whole input area is used to draw the contents.
    pub fn draw(
        &mut self,
        f: &mut ratatui::prelude::Frame,
        r: ratatui::prelude::Rect,
        is_playground: bool,
    ) {
        let input = Paragraph::new(self.input.as_str()).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Enter instruction:"),
        );
        let area = if is_playground {
            r
        } else {
            super::centered_rect(43, 40, None, r)
        };
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(3)])
            .margin(1)
            .split(area);
        // clear background
        f.render_widget(Clear, area);
        // render surrounding block
        let outer_block_title = if is_playground {
            "Playground mode"
        } else {
            "Run custom instruction"
        };
        let outer_block = Block::default()
            .title(outer_block_title)
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(self.theme.custom_instruction())
            .style(self.theme.single_instruction_block());
        f.render_widget(outer_block, area);
        f.render_widget(input, chunks[0]);
        f.set_cursor_position((
            chunks[0].x + self.cursor_position as u16 + 1,
            chunks[0].y + 1,
        ));
        // setup list
        let items_to_display = self.items_to_display();
        let possible_items = List::new(items_to_display)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("History")
                    .title_alignment(Alignment::Left)
                    .style(Style::default()),
            )
            .style(Style::default())
            .highlight_style(self.theme.list_item_highlight(false));
        // render list
        f.render_stateful_widget(possible_items, chunks[1], &mut self.allowed_values_state);
    }

    pub fn items_to_display(&self) -> Vec<String> {
        // Trie can not be set as variable in the struct, because it does not implement PartialEq
        let mut builder = TrieBuilder::new();
        for instruction in &self.executed_instructions {
            builder.push(instruction.trim());
        }

        if self.input.is_empty() {
            let mut to_display = self.executed_instructions.clone();
            to_display.reverse();
            return to_display;
        }
        let trie = builder.build();
        trie.predictive_search(self.input.as_bytes()).collect()
    }
}
