use ratatui::{
    prelude::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::Text,
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};
use text_align::TextAlign;

use super::{
    App, State, BREAKPOINT_ACCENT_COLOR, CODE_AREA_DEFAULT_COLOR, ERROR_COLOR,
    EXECUTION_FINISHED_POPUP_COLOR, LIST_ITEM_HIGHLIGHT_COLOR, NEXT_INSTRUCTION_BLOCK_BORDER_FG,
};

/// Draw the ui
#[allow(clippy::too_many_lines)]
pub fn draw(f: &mut Frame, app: &mut App) {
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

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(5),
                Constraint::Percentage(65),
                Constraint::Percentage(20),
                Constraint::Percentage(10),
            ]
            .as_ref(),
        )
        .split(global_chunks[0]);

    // draw keybinding hints
    f.render_widget(keybinding_hints, global_chunks[1]);

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Fill(1),
            Constraint::Length(3),
        ])
        .split(chunks[2]);

    let mut stack_chunks_constraints = vec![Constraint::Fill(1)];
    if app.show_call_stack {
        stack_chunks_constraints.push(Constraint::Percentage(30));
    }
    let stack_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(stack_chunks_constraints)
        .split(chunks[3]);

    // Code area
    let mut code_area = Block::default()
        .borders(Borders::ALL)
        .title_alignment(Alignment::Left)
        .border_type(BorderType::Rounded);
    if let State::RuntimeError(_) = app.state {
        code_area = code_area.border_style(Style::default().fg(ERROR_COLOR));
    } else if let State::DebugSelect(_, _) = app.state {
        code_area = code_area
            .border_style(Style::default().fg(BREAKPOINT_ACCENT_COLOR))
            .title("Debug select mode");
    } else {
        code_area = code_area
            .border_style(Style::default().fg(CODE_AREA_DEFAULT_COLOR))
            .title(format!("File: {}", app.filename.clone()));
    }

    // Create a List from all instructions and highlight current instruction
    let items = List::new(app.instruction_list_states.as_list_items())
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
        .highlight_symbol(">> ");

    // We can now render the item list
    f.render_stateful_widget(
        items,
        chunks[1],
        app.instruction_list_states.instruction_list_state_mut(),
    );

    // Breakpoint list
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
    //f.render_widget(breakpoints, chunks[0]);

    // Accumulator block
    let accumulator = Block::default()
        .borders(Borders::ALL)
        .title("Accumulators")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded);
    let accumulator_list =
        List::new(app.memory_lists_manager.accumulator_list()).block(accumulator);
    f.render_widget(accumulator_list, right_chunks[0]);

    // Memory cell block
    let memory_cells = Block::default()
        .borders(Borders::ALL)
        .title("Memory cells")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded);
    let memory_cell_list =
        List::new(app.memory_lists_manager.memory_cell_list()).block(memory_cells);
    f.render_widget(memory_cell_list, right_chunks[1]);

    // Next instruction block
    let next_instruction_block = Block::default()
        .borders(Borders::ALL)
        .title("Next instruction")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(NEXT_INSTRUCTION_BLOCK_BORDER_FG));
    let next_instruction = Paragraph::new(format!("{}", app.runtime.next_instruction_index() + 1))
        .block(next_instruction_block);
    f.render_widget(next_instruction, right_chunks[2]);

    // Stack block
    let stack = Block::default()
        .borders(Borders::ALL)
        .title("Stack")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded);
    let stack_list = List::new(app.memory_lists_manager.stack_list()).block(stack);
    f.render_widget(stack_list, stack_chunks[0]);

    // Render call stack if enabled
    if app.show_call_stack {
        let call_stack_block = Block::default()
            .borders(Borders::ALL)
            .title("Call Stack")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(NEXT_INSTRUCTION_BLOCK_BORDER_FG));
        let call_stack = List::new(app.runtime.call_stack_list()).block(call_stack_block);
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
    if let State::RuntimeError(e) = &app.state {
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
    if let State::CustomInstructionError(reason) = &app.state {
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

    // Draw custom instruction popup if it is active
    if let State::CustomInstruction(single_instruction) = &mut app.state {
        single_instruction.draw(f, global_chunks[0])
    }
}
