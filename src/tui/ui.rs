use ratatui::{Frame, layout::{Alignment, Constraint, Direction, Layout}, style::{Color, Modifier, Style}, text::{Line, Span}, widgets::{Block, Borders, List, ListItem, Paragraph}};

use crate::{model::{Priority, Todo}, tui::app::{self, InputMode}};


pub fn draw(frame: &mut Frame, app: &app::UiApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(frame.size());

    // Title
    let title = Paragraph::new("📝 Todo TUI")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    frame.render_widget(title, chunks[0]);

    let items: Vec<ListItem> = app
        .todos
        .iter()
        .enumerate()
        .map(|(index, todo)| {
            let checkbox = if todo.completed { "✓" } else { "☐" };
            let priority_color = match todo.priority {
                Priority::Low => Color::Green,
                Priority::Medium => Color::Yellow,
                Priority::High => Color::Red,
            };

            let style = if Some(index) == app.state.selected() {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let content = Line::from(vec![
                Span::styled(format!("{} ", checkbox), Style::default().fg(priority_color)),
                Span::styled(
                    todo.title.clone(),
                    if todo.completed {
                        Style::default().add_modifier(Modifier::CROSSED_OUT)
                    } else {
                        Style::default()
                    },
                ),
                Span::styled(
                    format!(" [{}]", todo.priority.as_str()),
                    Style::default().fg(priority_color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    todo.tags.as_ref().map(|t| format!(" [{}]", t.join(", "))).unwrap_or_default(),
                    Style::default().fg(Color::Blue),
                ),
            ]);

            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().title(format!(" Todos ({}) ", app.todos.len())).borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    let mut state = app.state.clone();
    frame.render_stateful_widget(list, chunks[1], &mut state);

    // Status bar
    let status_text = match app.mode {
        InputMode::Normal => {
            // let filter_info = app.filter_tag().map(|t| format!(" | Filter: {}", t)).unwrap_or_default();
            // let show_info = if app.show_completed() { " | All" } else { "" };
            // let msg = app.message().map(|m| format!(" | {}", m)).unwrap_or_default();
            format!(
                "a:add  space:toggle  d:delete  e:priority  f:filter  c:all  r:reload  q:quit  {}",
                /*filter_info, show_info, msg,*/ " ".repeat(50)
            )
        }
        _ => format!("{}{}", app.input_prompt, app.input_buffer),
    };

    let status_style = match app.mode {
        InputMode::Normal => Style::default().fg(Color::Gray),
        // InputMode::ConfirmingDelete => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        _ => Style::default().fg(Color::Yellow),
    };

    let status = Paragraph::new(status_text)
        .block(Block::default().borders(Borders::ALL))
        .style(status_style);
    frame.render_widget(status, chunks[2]);
}