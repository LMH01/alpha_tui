use ratatui::{
    prelude::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::Text,
    widgets::{Block, BorderType, Borders, Clear, List, ListDirection, ListItem, Paragraph},
    Frame,
};
use text_align::TextAlign;

use super::{
    run_instruction::SingleInstruction, App, State, BREAKPOINT_ACCENT_COLOR,
    CODE_AREA_DEFAULT_COLOR, ERROR_COLOR, EXECUTION_FINISHED_POPUP_COLOR,
    INTERNAL_MEMORY_BLOCK_BORDER_FG, LIST_ITEM_HIGHLIGHT_COLOR, MEMORY_BLOCK_BORDER_FG,
};

/// Draw the ui
#[allow(clippy::too_many_lines)]
pub fn draw(f: &mut Frame, app: &mut App) {
    // when the app is in sandbox mode, some things are rendered differently
    let is_sandbox = match app.state {
        State::Sandbox(_) => true,
        State::RuntimeError(_, is_sandbox) => is_sandbox,
        State::CustomInstructionError(_, is_sandbox) => is_sandbox,
        _ => false,
    };

    let (keybinding_hints, keybinding_hints_height) = app
        .keybinding_hints
        .keybinding_hint_paragraph(f.size().width);

    let global_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(keybinding_hints_height),
        ])
        .split(f.size());

    let mut chunk_constraints = if is_sandbox {
        // don't add chunk for breakpoints, when in sandbox mode
        Vec::new()
    } else {
        vec![Constraint::Length(5)]
    };
    chunk_constraints.push(Constraint::Percentage(65));
    chunk_constraints.push(Constraint::Percentage(20));
    chunk_constraints.push(Constraint::Percentage(10));
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(chunk_constraints)
        .split(global_chunks[0]);

    // draw keybinding hints
    f.render_widget(keybinding_hints, global_chunks[1]);

    let mut right_chunk_constraints = vec![Constraint::Percentage(30), Constraint::Fill(1)];
    if !is_sandbox {
        right_chunk_constraints.push(Constraint::Length(3))
    }
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(right_chunk_constraints)
        .split(chunks[if is_sandbox { 1 } else { 2 }]);

    let mut stack_chunks_constraints = vec![Constraint::Fill(1)];
    if app.show_call_stack {
        stack_chunks_constraints.push(Constraint::Percentage(30));
    }
    let stack_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(stack_chunks_constraints)
        .split(chunks[if is_sandbox { 2 } else { 3 }]);

    // central big part
    let mut central_constraints = vec![Constraint::Fill(1)];
    if is_sandbox {
        central_constraints.push(Constraint::Percentage(30));
    }
    let central_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(central_constraints)
        .split(chunks[if is_sandbox { 0 } else { 1 }]);

    // Code area
    let mut code_area = Block::default()
        .borders(Borders::ALL)
        .title_alignment(if is_sandbox {
            Alignment::Center
        } else {
            Alignment::Left
        })
        .border_type(BorderType::Rounded);
    if let State::RuntimeError(_, _) = app.state {
        code_area = code_area.border_style(Style::default().fg(ERROR_COLOR));
    } else if let State::DebugSelect(_, _) = app.state {
        code_area = code_area
            .border_style(Style::default().fg(BREAKPOINT_ACCENT_COLOR))
            .title("Debug select mode");
    } else {
        code_area = code_area
            .border_style(Style::default().fg(CODE_AREA_DEFAULT_COLOR))
            .title(if is_sandbox {
                format!("Executed instructions")
            } else {
                format!("File: {}", app.filename.clone())
            });
    }

    // Create a List from all instructions and highlight current instruction
    let items = List::new(app.instruction_list_states.as_list_items(is_sandbox))
        .block(code_area)
        .highlight_style(if let State::DebugSelect(_, _) = app.state {
            Style::default()
                .bg(BREAKPOINT_ACCENT_COLOR)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .bg(LIST_ITEM_HIGHLIGHT_COLOR)
                .add_modifier(Modifier::BOLD)
        })
        .highlight_symbol(">> ")
        .direction(if is_sandbox {
            ListDirection::BottomToTop
        } else {
            ListDirection::TopToBottom
        });

    // We can now render the item list
    f.render_stateful_widget(
        items,
        central_chunks[0],
        app.instruction_list_states.instruction_list_state_mut(),
    );

    // Breakpoint list
    if !is_sandbox {
        // don't render breakpoint list, if we are in sandbox mode
        let breakpoint_area = Block::default()
            .borders(Borders::ALL)
            .title("BPs")
            .border_style(Style::default().fg(BREAKPOINT_ACCENT_COLOR))
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded);

        // Create the items for the list
        let breakpoint_list_items: Vec<ListItem> = app
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
                    Style::default().fg(BREAKPOINT_ACCENT_COLOR),
                ))
            })
            .collect();

        // Create the list itself
        let breakpoints = List::new(breakpoint_list_items).block(breakpoint_area);

        f.render_stateful_widget(
            breakpoints,
            chunks[0],
            app.instruction_list_states.breakpoint_list_state_mut(),
        );
    }

    // Accumulator block
    let accumulator = Block::default()
        .borders(Borders::ALL)
        .title("Accumulators")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(MEMORY_BLOCK_BORDER_FG));
    let accumulator_list =
        List::new(app.memory_lists_manager.accumulator_list()).block(accumulator);
    f.render_widget(accumulator_list, right_chunks[0]);

    // Memory cell block
    let memory_cells = Block::default()
        .borders(Borders::ALL)
        .title("Memory cells")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(MEMORY_BLOCK_BORDER_FG));
    let memory_cell_list =
        List::new(app.memory_lists_manager.memory_cell_list()).block(memory_cells);
    f.render_widget(memory_cell_list, right_chunks[1]);

    // Next instruction block
    if !is_sandbox {
        // draw next instruction block only, if no in sandbox mode
        let next_instruction_title = if right_chunks[2].width >= 18 {
            "Next instruction"
        } else {
            "Next instr."
        };
        let next_instruction_block = Block::default()
            .borders(Borders::ALL)
            .title(next_instruction_title)
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(INTERNAL_MEMORY_BLOCK_BORDER_FG));
        let next_instruction =
            Paragraph::new(format!("{}", app.runtime.next_instruction_index() + 1))
                .block(next_instruction_block);
        f.render_widget(next_instruction, right_chunks[2]);
    }

    // Stack block
    let stack = Block::default()
        .borders(Borders::ALL)
        .title("Stack")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(MEMORY_BLOCK_BORDER_FG));
    let stack_list = List::new(app.memory_lists_manager.stack_list()).block(stack);
    f.render_widget(stack_list, stack_chunks[0]);

    // Render call stack if enabled
    if app.show_call_stack {
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
            .border_style(Style::default().fg(INTERNAL_MEMORY_BLOCK_BORDER_FG));
        let call_stack =
            List::new(app.memory_lists_manager.call_stack_list()).block(call_stack_block);
        f.render_widget(call_stack, stack_chunks[1]);
    }

    // Popup if execution has finished
    if app.state == State::Finished(true) {
        let block = Block::default()
            .title("Execution finished!")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(EXECUTION_FINISHED_POPUP_COLOR));
        let area = super::centered_rect(60, 20, None, f.size());
        let text = Paragraph::new(
            "Press [q] to exit.\nPress [t] to reset to start.\nPress [d] to dismiss this message.",
        )
        .block(block);
        f.render_widget(Clear, area); //this clears out the background
        f.render_widget(text, area);
    }

    // Popup if runtime error
    if let State::RuntimeError(e, _) = &app.state {
        let block = Block::default()
            .title("Runtime error!")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(ERROR_COLOR));
        let area = super::centered_rect(60, 30, None, f.size());
        let text = Paragraph::new(format!(
                "Execution can not continue due to the following problem:\n{}\n\nPress [q] to exit and to view further information regarding this error.\nPress [t] to reset to start.",
                e.reason)).block(block);
        f.render_widget(Clear, area); //this clears out the background
        f.render_widget(text, area);
    }

    // Draw error when instruction could not be parsed
    if let State::CustomInstructionError(reason, _) = &app.state {
        let block = Block::default()
            .title("Error: unable to parse instruction".to_string())
            .borders(Borders::ALL)
            .border_style(Style::default().fg(ERROR_COLOR));
        let area = super::centered_rect(60, 30, Some(6), f.size());
        let text = Paragraph::new(format!(
            "{}\n\nPress [q] to exit and to view further information regarding this error.\nPress [ENTER] to close.",
            reason
        ))
        .block(block);
        f.render_widget(Clear, area); //this clears out the background
        f.render_widget(text, area);
    }

    // Draw custom instruction popup/window
    if let State::CustomInstruction(single_instruction) = &mut app.state {
        single_instruction.draw(f, global_chunks[0], true)
    }
    match &mut app.state {
        State::Sandbox(single_instruction) => {
            single_instruction.draw(f, central_chunks[1], false);
        }
        State::CustomInstructionError(_, true) | State::RuntimeError(_, true) => {
            SingleInstruction::new(&app.executed_custom_instructions).draw(
                f,
                central_chunks[1],
                false,
            );
        }
        _ => (),
    }
}
