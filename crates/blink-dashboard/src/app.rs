use std::path::PathBuf;
use std::time::Instant;

use blink_analyzer::{compute_health, AnalysisReport, Analyzer, HealthReport};
use blink_core::{Project, ProjectDetector};

/// All state the dashboard renders. Every field is populated from a real
/// scan/analysis run against `root` — nothing here is placeholder data.
pub struct App {
    pub root: PathBuf,
    pub project: Option<Project>,
    pub analysis: Option<AnalysisReport>,
    pub health: Option<HealthReport>,
    pub online: bool,
    pub last_refreshed: Option<Instant>,
    pub last_refresh_ms: Option<u128>,
    pub status: String,
    pub should_quit: bool,
}

impl App {
    pub fn new(root: PathBuf) -> Self {
        let mut app = Self {
            root,
            project: None,
            analysis: None,
            health: None,
            online: false,
            last_refreshed: None,
            last_refresh_ms: None,
            status: "Loading...".to_string(),
            should_quit: false,
        };
        app.refresh();
        app
    }

    /// Re-detect the project and re-run analysis. Called on startup, on a
    /// manual refresh keypress, and whenever the file watcher reports a
    /// change.
    pub fn refresh(&mut self) {
        let start = Instant::now();

        match ProjectDetector::new().detect(&self.root) {
            Ok(project) => {
                let analysis = Analyzer::new()
                    .online(self.online)
                    .analyze(&project, &self.root);
                let health = compute_health(&analysis, &self.root);
                self.status = format!("Ready ({})", if self.online { "online" } else { "offline" });
                self.project = Some(project);
                self.analysis = Some(analysis);
                self.health = Some(health);
            }
            Err(err) => {
                self.status = format!("No project detected: {err}");
                self.project = None;
                self.analysis = None;
                self.health = None;
            }
        }

        self.last_refreshed = Some(Instant::now());
        self.last_refresh_ms = Some(start.elapsed().as_millis());
    }

    pub fn toggle_online(&mut self) {
        self.online = !self.online;
        self.refresh();
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}
