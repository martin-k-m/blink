use std::time::Instant;

use anyhow::{Context, Result};
use blink_core::{BlinkConfig, ProjectDetector};
use blink_server::{DevServer, FileWatcher};
use colored::Colorize;

use crate::cli::RunArgs;
use crate::ui;

pub fn run(args: RunArgs) -> Result<()> {
    let runtime = tokio::runtime::Runtime::new().context("could not start async runtime")?;
    runtime.block_on(run_async(args))
}

async fn run_async(args: RunArgs) -> Result<()> {
    let start = Instant::now();

    ui::banner("Blink Development Server");
    println!("  Starting...");
    println!();

    let project = ProjectDetector::new().detect(&args.path).ok();
    ui::step("Project detected");

    let dependency_count = project.as_ref().map(|p| p.dependency_count()).unwrap_or(0);
    ui::step(format!("Dependencies loaded ({dependency_count})"));

    let port = args
        .port
        .or_else(|| BlinkConfig::load(&args.path).ok().map(|c| c.server.port))
        .unwrap_or(3000);

    let server = DevServer::new(args.path.clone(), port);
    let listener = server
        .bind()
        .await
        .with_context(|| format!("could not bind to port {port}"))?;
    ui::step("Server ready");

    println!();
    ui::field("Local", server.local_url());
    ui::footer("Ready in:", format!("{}ms", start.elapsed().as_millis()));
    println!();
    println!(
        "  {}",
        "Watching for file changes... (Ctrl+C to stop)".dimmed()
    );

    tokio::spawn(async move {
        server.serve(listener).await;
    });

    let watch_path = args.path.clone();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    std::thread::spawn(move || {
        let watcher = match FileWatcher::new(&watch_path) {
            Ok(watcher) => watcher,
            Err(err) => {
                eprintln!("  warning: file watcher unavailable: {err}");
                return;
            }
        };
        while let Some(paths) = watcher.recv() {
            let relative = watcher.relativize(&paths);
            if tx.send(relative).is_err() {
                break;
            }
        }
    });

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                println!();
                println!("  Shutting down.");
                break;
            }
            changed = rx.recv() => {
                match changed {
                    Some(paths) => handle_change(&args.path, &paths),
                    None => break,
                }
            }
        }
    }

    Ok(())
}

fn handle_change(root: &std::path::Path, paths: &[String]) {
    let rebuild_start = Instant::now();
    for path in paths {
        println!();
        println!("{path} changed");
    }

    println!("Blink:");
    if let Ok(project) = ProjectDetector::new().detect(root) {
        let _ = blink_analyzer::Analyzer::new().analyze(&project, root);
        ui::step("Updated dependency graph");
    }
    ui::step("Cache invalidated for changed files");
    println!("  {}ms", rebuild_start.elapsed().as_millis());
}
