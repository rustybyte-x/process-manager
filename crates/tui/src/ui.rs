use process_manager_core::{
    app::{App, SortMode, StatusLevel, UiMode},
    process::ProcessStatus,
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, TableState, Wrap},
};

/// Renders the complete user interface for the current application state.
pub fn render(frame: &mut Frame, app: &App) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(12),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(frame.area());

    frame.render_widget(
        Paragraph::new("process-manager")
            .block(Block::default().borders(Borders::ALL).title("Title")),
        outer[0],
    );

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(72), Constraint::Percentage(28)])
        .split(outer[1]);

    render_process_table(frame, app, body[0]);
    render_details(frame, app, body[1]);
    render_input(frame, app, outer[2]);
    render_status(frame, app, outer[3]);

    if app.mode == UiMode::ConfirmKill {
        render_kill_dialog(frame, app);
    }
}

/// Renders the process table including the active sort marker and row selection.
fn render_process_table(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec![
        Cell::from(column_title("PID", app.sort_mode == SortMode::PidAsc)),
        Cell::from(column_title("NAME", app.sort_mode == SortMode::NameAsc)),
        Cell::from(column_title("CPU %", app.sort_mode == SortMode::CpuDesc)),
        Cell::from(column_title(
            "MEM MB",
            app.sort_mode == SortMode::MemoryDesc,
        )),
        Cell::from("STATUS"),
    ])
    .style(Style::default().add_modifier(Modifier::BOLD));

    let rows = app.filtered_indices.iter().map(|&index| {
        let process = &app.processes[index];

        Row::new(vec![
            Cell::from(process.pid.to_string()),
            Cell::from(truncate(&process.name, 26)),
            Cell::from(format!("{:.1}", process.cpu_percent)).style(cpu_style(process.cpu_percent)),
            Cell::from(format_memory_mb(process.memory_bytes)),
            Cell::from(format_status(process.status)).style(status_style(process.status)),
        ])
    });

    let table = Table::new(
        rows,
        [
            Constraint::Length(8),
            Constraint::Min(20),
            Constraint::Length(8),
            Constraint::Length(12),
            Constraint::Length(12),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title("Processes"))
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    let mut state = TableState::default();
    if !app.filtered_indices.is_empty() {
        state.select(Some(app.selected));
    }

    frame.render_stateful_widget(table, area, &mut state);
}

/// Renders the detail panel for the currently selected process.
fn render_details(frame: &mut Frame, app: &App, area: Rect) {
    let content = if let Some(process) = app.selected_process() {
        format!(
            "Name: {}\nPID: {}\nCPU: {:.1} %\nMemory: {} MB\nStatus: {}\n\nSort: {}\nFilter: {}\nAuto refresh: {}",
            process.name,
            process.pid,
            process.cpu_percent,
            format_memory_mb(process.memory_bytes),
            format_status(process.status),
            format_sort_mode(app.sort_mode),
            if app.filter_input.is_empty() {
                "-"
            } else {
                app.filter_input.as_str()
            },
            if app.auto_refresh { "on" } else { "off" },
        )
    } else {
        format!(
            "No process selected\n\nSort: {}\nFilter: {}\nAuto refresh: {}",
            format_sort_mode(app.sort_mode),
            if app.filter_input.is_empty() {
                "-"
            } else {
                app.filter_input.as_str()
            },
            if app.auto_refresh { "on" } else { "off" },
        )
    };

    let widget = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title("Details"))
        .wrap(Wrap { trim: true });

    frame.render_widget(widget, area);
}

/// Renders the current input area or mode hint.
fn render_input(frame: &mut Frame, app: &App, area: Rect) {
    let (title, content) = match app.mode {
        UiMode::Normal => (
            "Input",
            "Press / to filter, : to start a process, d to kill selected, s to sort",
        ),
        UiMode::Filter => ("Filter", app.filter_input.as_str()),
        UiMode::Command => ("Command", app.command_input.as_str()),
        UiMode::ConfirmKill => ("Input", "Confirm dialog is active"),
    };

    let widget = Paragraph::new(content).block(Block::default().borders(Borders::ALL).title(title));

    frame.render_widget(widget, area);
}

/// Renders the status bar using a color derived from the current status level.
fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let style = match app.status_level {
        StatusLevel::Info => Style::default(),
        StatusLevel::Success => Style::default().green(),
        StatusLevel::Warning => Style::default().yellow(),
        StatusLevel::Error => Style::default().red(),
    };

    let widget = Paragraph::new(app.status.as_str())
        .style(style)
        .block(Block::default().borders(Borders::ALL).title("Status"));

    frame.render_widget(widget, area);
}

/// Renders the centered kill confirmation dialog above the main layout.
fn render_kill_dialog(frame: &mut Frame, app: &App) {
    let area = centered_rect(60, 22, frame.area());

    let text = if let Some(process) = app.selected_process() {
        format!(
            "Kill process?\n\nName: {}\nPID: {}\n\nPress y to confirm, n to cancel",
            process.name, process.pid
        )
    } else {
        "No process selected".to_string()
    };

    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("Confirm Kill"))
            .wrap(Wrap { trim: true }),
        area,
    );
}

/// Returns a rectangle centered within `area` using percentage-based sizing.
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .flex(Flex::Center)
        .split(vertical[1])[1]
}

/// Returns a column title with a simple active-sort marker.
fn column_title(title: &str, active: bool) -> String {
    if active {
        format!("{title} *")
    } else {
        title.to_string()
    }
}

/// Returns a style for CPU values based on simple usage thresholds.
fn cpu_style(cpu: f32) -> Style {
    if cpu >= 70.0 {
        Style::default().fg(Color::Red)
    } else if cpu >= 30.0 {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Green)
    }
}

/// Returns a style for displaying the given process status.
fn status_style(status: ProcessStatus) -> Style {
    match status {
        ProcessStatus::Running => Style::default().fg(Color::Green),
        ProcessStatus::Sleeping => Style::default().fg(Color::Yellow),
        ProcessStatus::Stopped => Style::default().fg(Color::Red),
        ProcessStatus::Zombie => Style::default().fg(Color::Magenta),
        ProcessStatus::Unknown => Style::default().fg(Color::Gray),
    }
}

/// Formats a process status as a human-readable label.
fn format_status(status: ProcessStatus) -> &'static str {
    match status {
        ProcessStatus::Running => "Running",
        ProcessStatus::Sleeping => "Sleeping",
        ProcessStatus::Stopped => "Stopped",
        ProcessStatus::Zombie => "Zombie",
        ProcessStatus::Unknown => "Unknown",
    }
}

/// Formats a memory value as megabytes for display.
fn format_memory_mb(bytes: u64) -> String {
    let mb = bytes as f64 / 1024.0 / 1024.0;
    format!("{mb:.1}")
}

/// Truncates a string to a maximum character count and appends an ellipsis if needed.
fn truncate(value: &str, max_len: usize) -> String {
    let mut chars = value.chars();
    let collected: String = chars.by_ref().take(max_len).collect();

    if value.chars().count() > max_len {
        format!("{collected}…")
    } else {
        collected
    }
}

/// Formats the active sort mode as a human-readable label.
fn format_sort_mode(sort_mode: SortMode) -> &'static str {
    match sort_mode {
        SortMode::NameAsc => "Name",
        SortMode::PidAsc => "PID",
        SortMode::CpuDesc => "CPU",
        SortMode::MemoryDesc => "Memory",
    }
}
