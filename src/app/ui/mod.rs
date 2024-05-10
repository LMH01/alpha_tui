use ratatui::{
    prelude::{Alignment, Constraint, Direction, Layout},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Clear, List, ListDirection, ListItem, Paragraph},
    Frame,
};
use text_align::TextAlign;

use super::{keybindings::KeySymbol, run_instruction::SingleInstruction, App, State};

pub mod style;
pub mod syntax_highlighting;

impl App {
    /// Draw the ui
    #[allow(clippy::too_many_lines)]
    pub fn draw(&mut self, f: &mut Frame) {
        // when the app is in playground mode, some things are rendered differently
        let is_playground = match self.state {
            State::Playground(_) => true,
            State::RuntimeError(_, is_playground) => is_playground,
            State::CustomInstructionError(_, is_playground) => is_playground,
            _ => false,
        };

        let (keybinding_hints, keybinding_hints_height) = self
            .keybinding_hints
            .keybinding_hint_paragraph(f.size().width);

        let global_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(keybinding_hints_height),
            ])
            .split(f.size());

        let mut chunk_constraints = if is_playground {
            // don't add chunk for breakpoints, when in playground mode
            Vec::new()
        } else {
            vec![Constraint::Length(5)]
        };
        chunk_constraints.push(Constraint::Fill(1));
        chunk_constraints.push(if global_chunks[0].width < 49 {
            Constraint::Length(10)
        } else {
            Constraint::Percentage(20)
        });
        chunk_constraints.push(Constraint::Percentage(10));
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(chunk_constraints)
            .split(global_chunks[0]);

        // draw keybinding hints
        f.render_widget(keybinding_hints, global_chunks[1]);

        let mut right_chunk_constraints = vec![Constraint::Percentage(30), Constraint::Fill(1)];
        if !is_playground {
            right_chunk_constraints.push(Constraint::Length(3))
        }
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(right_chunk_constraints)
            .split(chunks[if is_playground { 1 } else { 2 }]);

        let mut stack_chunks_constraints = vec![Constraint::Fill(1)];
        if self.show_call_stack {
            stack_chunks_constraints.push(Constraint::Percentage(30));
        }
        let stack_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(stack_chunks_constraints)
            .split(chunks[if is_playground { 2 } else { 3 }]);

        // central big part
        let central_constraints = if is_playground {
            vec![Constraint::Percentage(60), Constraint::Min(8)]
        } else {
            vec![Constraint::Fill(1)]
        };
        let central_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(central_constraints)
            .split(chunks[if is_playground { 0 } else { 1 }]);

        // Code area
        let mut code_area = Block::default()
            .borders(Borders::ALL)
            .title_alignment(if is_playground {
                Alignment::Center
            } else {
                Alignment::Left
            })
            .border_type(BorderType::Rounded)
            .style(self.theme.code_block());
        if let State::RuntimeError(_, false) = self.state {
            code_area = code_area.border_style(self.theme.error_block_border());
        } else if let State::DebugSelect(_, _) = self.state {
            code_area = code_area
                .border_style(self.theme.breakpoint_border())
                .title("Debug select mode");
        } else {
            code_area = code_area
                .border_style(self.theme.code_block_border())
                .title(if is_playground {
                    "Executed instructions".to_string()
                } else {
                    format!("File: {}", self.filename.clone())
                });
        }

        // Create a List from all instructions and highlight current instruction
        let items = List::new(self.instruction_list_states.as_list_items(is_playground))
            .block(code_area)
            .highlight_style(if let State::DebugSelect(_, _) = self.state {
                self.theme.list_item_highlight(true)
            } else {
                self.theme.list_item_highlight(false)
            })
            .highlight_symbol(">> ")
            .direction(if is_playground {
                ListDirection::BottomToTop
            } else {
                ListDirection::TopToBottom
            });

        // We can now render the item list
        f.render_stateful_widget(
            items,
            central_chunks[0],
            self.instruction_list_states.instruction_list_state_mut(),
        );

        // Breakpoint list
        if !is_playground {
            // don't render breakpoint list, if we are in playground mode
            let breakpoint_area = Block::default()
                .borders(Borders::ALL)
                .title("BPs")
                .border_style(self.theme.breakpoint_border())
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded)
                .style(self.theme.breakpoint_block());

            // Create the items for the list
            let breakpoint_list_items: Vec<ListItem> = self
                .instruction_list_states
                .instructions()
                .iter()
                .map(|f| {
                    let v = if f.2 {
                        "*".to_string()
                    } else {
                        " ".to_string()
                    };
                    ListItem::new(Text::styled(
                        v.center_align(chunks[0].width.saturating_sub(2) as usize),
                        self.theme.breakpoint(),
                    ))
                })
                .collect();

            // Create the list itself
            let breakpoints = List::new(breakpoint_list_items).block(breakpoint_area);

            f.render_stateful_widget(
                breakpoints,
                chunks[0],
                self.instruction_list_states.breakpoint_list_state_mut(),
            );
        }

        // Accumulator block
        let accumulator_title = match right_chunks[0].width {
            0..=13 => "Accs",
            14..=u16::MAX => "Accumulators",
        };
        let accumulator = Block::default()
            .borders(Borders::ALL)
            .title(accumulator_title)
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded)
            .border_style(self.theme.memory_block_border())
            .style(self.theme.memory_block());
        let accumulator_list =
            List::new(self.memory_lists_manager.accumulator_list()).block(accumulator);
        f.render_widget(accumulator_list, right_chunks[0]);

        // Memory cell block
        let memory_cells_title = match right_chunks[1].width {
            0..=10 => "MCs",
            11..=13 => "Mem cells",
            14..=u16::MAX => "Memory cells",
        };
        let memory_cells = Block::default()
            .borders(Borders::ALL)
            .title(memory_cells_title)
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded)
            .border_style(self.theme.memory_block_border())
            .style(self.theme.memory_block());
        let memory_cell_list =
            List::new(self.memory_lists_manager.memory_cell_list()).block(memory_cells);
        f.render_widget(memory_cell_list, right_chunks[1]);

        // Next instruction block
        if !is_playground {
            // draw next instruction block only, if not in playground mode
            let next_instruction_title = match right_chunks[2].width {
                0..=17 => "Next instr.",
                18..=u16::MAX => "Next instruction",
            };
            let next_instruction_block = Block::default()
                .borders(Borders::ALL)
                .title(next_instruction_title)
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded)
                .border_style(self.theme.internal_memory_block_border())
                .style(self.theme.internal_memory_block());
            let next_instruction =
                Paragraph::new(format!("{}", self.runtime.next_instruction_index() + 1))
                    .block(next_instruction_block);
            f.render_widget(next_instruction, right_chunks[2]);
        }

        // Stack block
        let stack_title = match stack_chunks[0].width {
            0..=6 => "Stck",
            7..=u16::MAX => "Stack",
        };
        let stack = Block::default()
            .borders(Borders::ALL)
            .title(stack_title)
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded)
            .border_style(self.theme.memory_block_border())
            .style(self.theme.memory_block());
        let stack_list = List::new(self.memory_lists_manager.stack_list()).block(stack);
        f.render_widget(stack_list, stack_chunks[0]);

        // Render call stack if enabled
        if self.show_call_stack {
            let call_stack_title = if stack_chunks[1].width >= 12 {
                "Call Stack"
            } else {
                "CS"
            };
            let call_stack_block = Block::default()
                .borders(Borders::ALL)
                .title(call_stack_title)
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded)
                .border_style(self.theme.internal_memory_block_border())
                .style(self.theme.internal_memory_block());
            let call_stack =
                List::new(self.memory_lists_manager.call_stack_list()).block(call_stack_block);
            f.render_widget(call_stack, stack_chunks[1]);
        }

        // Popup if execution has finished
        if self.state == State::Finished(true) {
            let block = Block::default()
                .title("Execution finished!")
                .borders(Borders::ALL)
                .border_style(self.theme.execution_finished_popup_border())
                .style(self.theme.execution_finished_block());
            let area = super::centered_rect_abs(5, 36, f.size());
            let text = paragraph_with_line_wrap(
            format!("Press [t] to reset to start.\nPress [d] to dismiss this message.\nPress [q] or [{}] to exit.", KeySymbol::Escape.to_string()),
            area.width,
        )
        .block(block);
            f.render_widget(Clear, area); //this clears out the background
            f.render_widget(text, area);
        }

        // Draw custom instruction popup/window
        if let State::CustomInstruction(single_instruction) = &mut self.state {
            single_instruction.draw(f, global_chunks[0], false)
        }
        match &mut self.state {
            State::Playground(single_instruction) => {
                single_instruction.draw(f, central_chunks[1], true);
            }
            State::CustomInstructionError(_, true) | State::RuntimeError(_, true) => {
                SingleInstruction::new(&self.executed_custom_instructions, &self.theme).draw(
                    f,
                    central_chunks[1],
                    true,
                );
            }
            _ => (),
        }

        // Popup if runtime error
        if let State::RuntimeError(e, _) = &self.state {
            let block = Block::default()
                .title("Runtime error!")
                .borders(Borders::ALL)
                .border_style(self.theme.error_block_border())
                .style(self.theme.error_block());
            let area = super::centered_rect(60, 30, None, f.size());
            let text = paragraph_with_line_wrap(if is_playground {format!("This instruction could not be executed due to the following problem:\n{}\n\nPress [q] to exit and to view further information regarding this error.\nPress [ENTER] to close.", e.reason)} else {format!(
                "Execution can not continue due to the following problem:\n{}\n\nPress [q] or [{}] to exit and to view further information regarding this error.\nPress [t] to reset to start.",
                e.reason, KeySymbol::Escape.to_string())}, area.width - 2).block(block);
            f.render_widget(Clear, area); //this clears out the background
            f.render_widget(text, area);
        }

        // Draw error when instruction could not be parsed
        if let State::CustomInstructionError(reason, _) = &self.state {
            let block = Block::default()
                .title("Error: unable to parse instruction".to_string())
                .borders(Borders::ALL)
                .border_style(self.theme.error_block_border())
                .style(self.theme.error_block());
            let area = super::centered_rect(
                60,
                30,
                if f.size().width <= 124 {
                    Some(7)
                } else {
                    Some(6)
                },
                f.size(),
            );
            let text = paragraph_with_line_wrap(format!(
            "{}\n\nPress [q] or [{}] to exit and to view further information regarding this error.\nPress [ENTER] to close.",
            reason,
            KeySymbol::Escape.to_string()
        ), area.width)
        .block(block);
            f.render_widget(Clear, area); //this clears out the background
            f.render_widget(text, area);
        }

        // Draw error when custom instruction could not be build
        if let State::BuildProgramError(_) = &self.state {
            let block = Block::default()
                .title("Error: instruction forbidden".to_string())
                .borders(Borders::ALL)
                .border_style(self.theme.error_block_border())
                .style(self.theme.error_block());
            let area = super::centered_rect(
                60,
                30,
                if f.size().width <= 124 {
                    Some(7)
                } else {
                    Some(6)
                },
                f.size(),
            );
            let text = paragraph_with_line_wrap(format!(
            "The entered instruction is forbidden.\n\nPress [q] or [{}] to exit and to view further information regarding this error.\nPress [ENTER] to close.",
            KeySymbol::Escape.to_string()
        ), area.width)
        .block(block);
            f.render_widget(Clear, area); //this clears out the background
            f.render_widget(text, area);
        }
    }
}

/// Creates a paragraph from the input text, where a new line is created when the space is to little
/// to fit the whole text in one line.
fn paragraph_with_line_wrap(text: String, width: u16) -> Paragraph<'static> {
    let lines = text
        .split('\n')
        .map(|f| f.to_string())
        .collect::<Vec<String>>();
    let mut styled_lines = Vec::new();
    for line in lines {
        let mut styled_line = Vec::new();
        let words = line
            .split(' ')
            .map(|f| f.to_string())
            .collect::<Vec<String>>();
        let mut width_used = 0;
        for word in words {
            if word.len() + width_used > width as usize {
                styled_lines.push(Line::from(styled_line));
                styled_line = Vec::new();
                width_used = 0;
            }
            width_used += word.len() + 1;
            styled_line.push(Span::from(format!("{} ", word)));
        }
        if !styled_line.is_empty() {
            styled_lines.push(Line::from(styled_line));
        }
    }
    Paragraph::new(styled_lines)
}
