use ratatui::backend::TestBackend;
use ratatui::Terminal;
use tempfile::TempDir;

use crate::app::App;
use crate::ui;

fn rendered_buffer(root: std::path::PathBuf) -> String {
    let app = App::new(root);
    let backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|frame| ui::draw(frame, &app)).unwrap();

    let buffer = terminal.backend().buffer().clone();
    let mut text = String::new();
    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            text.push_str(buffer[(x, y)].symbol());
        }
        text.push('\n');
    }
    text
}

#[test]
fn renders_detected_project_stats() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"sample\"\n",
    )
    .unwrap();
    std::fs::write(dir.path().join("src.rs"), "fn main() {}").unwrap();

    let text = rendered_buffer(dir.path().to_path_buf());

    assert!(text.contains("sample"));
    assert!(text.contains("Health"));
    assert!(text.contains("Stats"));
    assert!(text.contains("refresh"));
}

#[test]
fn renders_placeholder_when_no_project_detected() {
    let dir = TempDir::new().unwrap();

    let text = rendered_buffer(dir.path().to_path_buf());

    assert!(text.contains("No project detected"));
}
