use blink_analyzer::format_bytes;
use blink_report::project_type_label;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Gauge, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::app::App;

const ACCENT: Color = Color::Rgb(255, 138, 0);

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(7),
            Constraint::Min(3),
            Constraint::Length(3),
        ])
        .split(area);

    draw_header(frame, chunks[0], app);
    draw_health(frame, chunks[1], app);
    draw_stats(frame, chunks[2], app);
    draw_issues(frame, chunks[3], app);
    draw_footer(frame, chunks[4], app);
}

fn draw_header(frame: &mut Frame, area: Rect, app: &App) {
    let title = match &app.project {
        Some(project) => format!(
            " \u{26a1} Blink Dashboard \u{2014} {} ({}) ",
            project.name,
            project_type_label(project)
        ),
        None => " \u{26a1} Blink Dashboard ".to_string(),
    };
    let block = Block::default().borders(Borders::ALL).title(Span::styled(
        title,
        Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
    ));
    frame.render_widget(block, area);
}

fn draw_health(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default().borders(Borders::ALL).title(" Health ");
    let score = app.health.as_ref().map(|h| h.overall).unwrap_or(0);
    let gauge = Gauge::default()
        .block(block)
        .gauge_style(Style::default().fg(ACCENT))
        .percent(u16::from(score))
        .label(format!("{score}%"));
    frame.render_widget(gauge, area);
}

fn draw_stats(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default().borders(Borders::ALL).title(" Stats ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(project) = &app.project else {
        frame.render_widget(Paragraph::new(app.status.as_str()), inner);
        return;
    };

    let transitive = app
        .analysis
        .as_ref()
        .and_then(|a| a.dependency_counts.transitive)
        .map(|n| n.to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let build_output = app
        .analysis
        .as_ref()
        .and_then(|a| a.build_output_bytes)
        .map(format_bytes)
        .unwrap_or_else(|| "not built yet".to_string());

    let lines = vec![
        Line::from(vec![
            Span::styled("Files: ", Style::default().add_modifier(Modifier::DIM)),
            Span::raw(project.file_count.to_string()),
            Span::raw("    "),
            Span::styled(
                "Direct deps: ",
                Style::default().add_modifier(Modifier::DIM),
            ),
            Span::raw(project.dependency_count().to_string()),
            Span::raw("    "),
            Span::styled("Transitive: ", Style::default().add_modifier(Modifier::DIM)),
            Span::raw(transitive),
        ]),
        Line::from(vec![
            Span::styled("Config: ", Style::default().add_modifier(Modifier::DIM)),
            Span::raw(project.config_file.clone()),
            Span::raw("    "),
            Span::styled(
                "Build output: ",
                Style::default().add_modifier(Modifier::DIM),
            ),
            Span::raw(build_output),
        ]),
        Line::from(vec![
            Span::styled(
                "Online checks: ",
                Style::default().add_modifier(Modifier::DIM),
            ),
            Span::raw(if app.online { "on" } else { "off" }),
            Span::raw("    "),
            Span::styled(
                "Last refresh: ",
                Style::default().add_modifier(Modifier::DIM),
            ),
            Span::raw(
                app.last_refresh_ms
                    .map(|ms| format!("{ms}ms"))
                    .unwrap_or_default(),
            ),
        ]),
    ];
    frame.render_widget(Paragraph::new(lines), inner);
}

fn draw_issues(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Issues & Suggestions ");

    let Some(analysis) = &app.analysis else {
        frame.render_widget(Paragraph::new(app.status.as_str()).block(block), area);
        return;
    };

    let issues = blink_report::issues(analysis);
    let recommendations = analysis.recommendations();

    let mut items: Vec<ListItem> = Vec::new();
    if issues.is_empty() && recommendations.is_empty() {
        items.push(ListItem::new(Span::styled(
            "No issues detected",
            Style::default().fg(Color::Green),
        )));
    } else {
        for issue in &issues {
            items.push(ListItem::new(Line::from(vec![
                Span::styled("\u{26a0} ", Style::default().fg(Color::Yellow)),
                Span::raw(issue.summary.clone()),
            ])));
        }
        for rec in &recommendations {
            items.push(ListItem::new(Line::from(vec![
                Span::styled("\u{2192} ", Style::default().fg(ACCENT)),
                Span::raw(rec.clone()),
            ])));
        }
    }

    frame.render_widget(List::new(items).block(block), area);
}

fn draw_footer(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default().borders(Borders::ALL);
    let text = Line::from(vec![
        Span::styled("[r]", Style::default().fg(ACCENT)),
        Span::raw(" refresh  "),
        Span::styled("[o]", Style::default().fg(ACCENT)),
        Span::raw(" toggle online checks  "),
        Span::styled("[q]", Style::default().fg(ACCENT)),
        Span::raw(" quit    "),
        Span::styled(
            app.status.as_str(),
            Style::default().add_modifier(Modifier::DIM),
        ),
    ]);
    frame.render_widget(Paragraph::new(text).block(block), area);
}
