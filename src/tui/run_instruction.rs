use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::Style,
    widgets::{Block, Borders, Clear, List, ListState, Paragraph},
};
use trie_rs::TrieBuilder;

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
}

impl SingleInstruction {
    /// Create a new single instruction.
    pub fn new(executed_instructions: &Vec<String>) -> Self {
        Self {
            input: String::new(),
            cursor_position: 0,
            allowed_values_state: ListState::default(),
            executed_instructions: executed_instructions.clone(),
        }
    }

    pub fn draw(&mut self, f: &mut ratatui::prelude::Frame, r: ratatui::prelude::Rect) {
        // TODO set styles correctly
        let input = Paragraph::new(self.input.as_str())
            .style(Style::default())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Enter instruction:"),
            );
        let area = super::centered_rect(60, 40, None, r);
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Fill(1)])
            .margin(1)
            .split(area);
        // clear background
        f.render_widget(Clear, area);
        // render surrounding block
        let outer_block = Block::default()
            .title("Run custom instruction")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL);
        f.render_widget(outer_block, area);
        f.render_widget(input, chunks[0]);
        f.set_cursor(
            chunks[0].x + self.cursor_position as u16 + 1,
            chunks[0].y + 1,
        );
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
            .highlight_style(Style::default());
        // render list
        f.render_stateful_widget(possible_items, chunks[1], &mut self.allowed_values_state)
    }

    fn items_to_display(&self) -> Vec<String> {
        // Trie can not be set as variable in the struct, because it does not implement PartialEq
        let mut builder = TrieBuilder::new();
        for instruction in &self.executed_instructions {
            builder.push(instruction);
        }

        if self.input.is_empty() {
            return self.executed_instructions.clone();
        }
        let trie = builder.build();
        trie.predictive_search(&self.input.as_bytes())
            .iter()
            .map(|u8s| {
                std::str::from_utf8(u8s)
                    .expect("Input value should be u8")
                    .to_string()
            })
            .collect()
    }
}
