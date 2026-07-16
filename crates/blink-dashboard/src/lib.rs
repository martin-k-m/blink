//! Blink's interactive terminal dashboard: a `ratatui` TUI showing a
//! project's real, live-refreshing scan/analysis/health state. Every
//! number on screen comes from actually re-running Blink's detection and
//! analysis engines — refreshed on a keypress or automatically when the
//! file watcher reports a change.

mod app;
mod ui;

#[cfg(test)]
mod tests;

use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

use anyhow::Result;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};

pub use app::App;

/// How long to wait for a keyboard event before redrawing anyway, so the
/// dashboard picks up file-watcher-triggered refreshes without waiting for
/// a keypress.
const POLL_INTERVAL: Duration = Duration::from_millis(200);

/// Run the dashboard against `root` until the user quits. Takes over the
/// terminal (raw mode + alternate screen) for the duration and always
/// restores it afterward, even on error.
pub fn run_dashboard(root: PathBuf) -> Result<()> {
    let mut terminal = ratatui::init();
    let mut app = App::new(root.clone());

    let (change_tx, change_rx) = mpsc::channel::<()>();
    std::thread::spawn(move || {
        let Ok(watcher) = blink_server::FileWatcher::new(&root) else {
            return;
        };
        while watcher.recv().is_some() {
            if change_tx.send(()).is_err() {
                break;
            }
        }
    });

    let result = event_loop(&mut terminal, &mut app, &change_rx);
    ratatui::restore();
    result
}

fn event_loop(
    terminal: &mut ratatui::DefaultTerminal,
    app: &mut App,
    change_rx: &mpsc::Receiver<()>,
) -> Result<()> {
    loop {
        if change_rx.try_recv().is_ok() {
            // Drain any additional queued events from a burst of saves so
            // one edit doesn't trigger a refresh per file.
            while change_rx.try_recv().is_ok() {}
            app.status = "File change detected \u{2014} refreshing...".to_string();
            app.refresh();
        }

        terminal.draw(|frame| ui::draw(frame, app))?;

        if event::poll(POLL_INTERVAL)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => app.quit(),
                        KeyCode::Char('r') => app.refresh(),
                        KeyCode::Char('o') => app.toggle_online(),
                        _ => {}
                    }
                }
            }
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}
