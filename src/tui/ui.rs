use ratatui::{
    prelude::{Alignment, Backend, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::Text,
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Tabs},
    Frame,
};
use text_align::TextAlign;

use super::{
    App, State, BREAKPOINT_ACCENT_COLOR, CODE_AREA_DEFAULT_COLOR, ERROR_COLOR,
    EXECUTION_FINISHED_POPUP_COLOR, KEY_HINTS_COLOR, LIST_ITEM_HIGHLIGHT_COLOR,
};

/// Draw the ui
#[allow(clippy::too_many_lines)]
pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let global_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(99), Constraint::Percentage(1)])
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

    // Key hints
    let key_hints = Tabs::new(app.active_keybind_hints())
        .block(Block::default().borders(Borders::NONE))
        .style(Style::default().fg(KEY_HINTS_COLOR));
    f.render_widget(key_hints, global_chunks[1]);

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(chunks[2]);

    // Code area
    let mut code_area = Block::default()
        .borders(Borders::ALL)
        .title_alignment(Alignment::Left)
        .border_type(BorderType::Rounded);
    if let State::Errored(_) = app.state {
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

    // Stack block
    let stack = Block::default()
        .borders(Borders::ALL)
        .title("Stack")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded);
    let stack_list = List::new(app.memory_lists_manager.stack_list()).block(stack);
    f.render_widget(stack_list, chunks[3]);

    // Popup if execution has finished
    if app.state == State::Finished(true) {
        let block = Block::default()
            .title("Execution finished!")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(EXECUTION_FINISHED_POPUP_COLOR));
        let area = centered_rect(60, 20, f.size());
        let text = Paragraph::new(
            "Press [q] to exit.\nPress [s] to reset to start.\nPress [d] to dismiss this message.",
        )
        .block(block);
        f.render_widget(Clear, area); //this clears out the background
        f.render_widget(text, area);
    }

    // Popup if runtime error
    if let State::Errored(e) = &app.state {
        let block = Block::default()
            .title("Runtime error!")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(ERROR_COLOR));
        let area = centered_rect(60, 30, f.size());
        let text = if app.jump_to_line_used {
            Paragraph::new(format!(
                "Execution can not continue due to the following problem:\n{}\n\nPress [q] to exit.\nPress [s] to reset to start.",
                e.reason)).block(block)
        } else {
            Paragraph::new(format!(
                "Execution can not continue due to the following problem:\n{}\n\nPress [q] to exit.",
                e.reason)).block(block)
        };
        f.render_widget(Clear, area); //this clears out the background
        f.render_widget(text, area);
    }
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`.
/// Copied from tui examples.
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
