use anyhow::{anyhow, Context, Result};
use blink_analyzer::{audit, AuditReport, AuditScope, AuditStatus, Finding, Severity};
use blink_core::ProjectDetector;
use colored::Colorize;

use crate::cli::SecurityArgs;
use crate::ui;

pub fn run(args: SecurityArgs) -> Result<()> {
    let project = ProjectDetector::new()
        .detect(&args.path)
        .with_context(|| format!("could not analyze {}", args.path.display()))?;

    let spinner =
        (!args.json).then(|| ui::spinner("Checking OSV.dev for known vulnerabilities..."));
    let report = audit(&project);
    if let Some(spinner) = &spinner {
        spinner.finish_and_clear();
    }

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        render(&report);
    }

    // A failed audit must not exit 0: "we couldn't check" is not "you're
    // fine", and a CI step that treats it as such is exactly the false
    // reassurance this command exists to avoid.
    if matches!(report.status, AuditStatus::SourceUnavailable { .. }) {
        // The reason is already on screen (or in the JSON), so this only has
        // to carry the non-zero exit code.
        return Err(anyhow!(
            "the security audit could not be completed — nothing was verified"
        ));
    }
    Ok(())
}

fn render(report: &AuditReport) {
    ui::banner("Blink Security");
    println!();

    match &report.status {
        AuditStatus::UnsupportedEcosystem { language } => {
            println!(
                "  {} No advisory database is wired up for {language} projects — nothing was checked.",
                "\u{2717}".red().bold()
            );
            println!();
            return;
        }
        AuditStatus::NothingToAudit => {
            println!(
                "  {} No resolved packages to audit (no dependencies and no lockfile).",
                "\u{2013}".dimmed()
            );
            println!();
            return;
        }
        AuditStatus::SourceUnavailable { reason } => {
            render_scope(report);
            println!();
            println!(
                "  {} Could not reach OSV.dev — {reason}",
                "\u{2717}".red().bold()
            );
            println!(
                "  {}",
                "This is NOT a clean result. Nothing was verified; re-run when you have network access."
                    .yellow()
            );
            println!();
            return;
        }
        AuditStatus::Completed => {}
    }

    render_scope(report);
    println!();

    if report.findings.is_empty() {
        println!(
            "  {} {}",
            "\u{2713}".green().bold(),
            "0 known advisories against anything audited.".bold()
        );
        render_coverage_note(report);
        render_source_note(report);
        return;
    }

    render_group(
        "Direct dependencies",
        &report.direct_findings().collect::<Vec<_>>(),
        report,
    );
    render_group(
        "Transitive dependencies",
        &report.transitive_findings().collect::<Vec<_>>(),
        report,
    );

    let advisories = report.distinct_advisories();
    let mut counts = [0usize; 4];
    let mut unrated = 0usize;
    for advisory in &advisories {
        match advisory.severity {
            Some(Severity::Critical) => counts[0] += 1,
            Some(Severity::High) => counts[1] += 1,
            Some(Severity::Moderate) => counts[2] += 1,
            Some(Severity::Low) => counts[3] += 1,
            None => unrated += 1,
        }
    }

    println!();
    println!(
        "  {} across {} of {} audited {} ({} critical, {} high, {} moderate, {} low, {} unrated)",
        format!(
            "{} advisor{}",
            advisories.len(),
            if advisories.len() == 1 { "y" } else { "ies" }
        )
        .bold(),
        report.findings.len(),
        report.audited,
        if report.audited == 1 {
            "package"
        } else {
            "packages"
        },
        counts[0],
        counts[1],
        counts[2],
        counts[3],
        unrated,
    );
    render_coverage_note(report);
    render_source_note(report);
}

fn render_scope(report: &AuditReport) {
    let ecosystem = report.ecosystem.unwrap_or("unknown");
    match &report.scope {
        AuditScope::Lockfile { file, .. } => ui::field(
            "Audited",
            format!(
                "{} resolved {} from {file} ({ecosystem})",
                ui::format_count(report.audited),
                if report.audited == 1 {
                    "package"
                } else {
                    "packages"
                },
            ),
        ),
        AuditScope::DeclaredOnly => ui::field(
            "Audited",
            format!(
                "{} declared {} ({ecosystem})",
                ui::format_count(report.audited),
                if report.audited == 1 {
                    "dependency"
                } else {
                    "dependencies"
                },
            ),
        ),
    }
}

fn render_group(title: &str, findings: &[&Finding], report: &AuditReport) {
    if findings.is_empty() {
        return;
    }

    println!("  {}", title.bold());
    for finding in findings {
        println!(
            "    {} {} {}{}",
            "\u{26a0}".yellow().bold(),
            finding.name.bold(),
            finding.version.dimmed(),
            provenance(finding, report),
        );
        for advisory in &finding.advisories {
            let severity = advisory
                .severity
                .map_or("unrated", |severity| severity.label());
            match &advisory.summary {
                Some(summary) => println!("        {} ({severity}) — {}", advisory.id, summary),
                None => println!("        {} ({severity})", advisory.id),
            }
        }
    }
    println!();
}

/// The parenthetical that says how this package got here. A path is only ever
/// shown when the lockfile actually recorded one.
fn provenance(finding: &Finding, report: &AuditReport) -> String {
    if finding.path.len() > 1 {
        return format!(
            " {}",
            format!("via {}", finding.path.join(" \u{2192} ")).dimmed()
        );
    }
    if finding.direct || !matches!(report.scope, AuditScope::Lockfile { .. }) {
        return String::new();
    }
    format!(" {}", "(dependency path unavailable)".dimmed())
}

/// Say plainly what the audit did and did not cover. The point of this
/// command is that a "clean" verdict is trustworthy, which means never
/// implying coverage that wasn't there.
fn render_coverage_note(report: &AuditReport) {
    match &report.scope {
        AuditScope::Lockfile {
            file,
            records_edges,
        } => {
            if !records_edges {
                println!();
                println!(
                    "  {}",
                    format!(
                        "Blink reads resolved versions but not dependency edges from {file}, \
                         so it can't show which declared dependency pulls each package in."
                    )
                    .dimmed()
                );
            }
        }
        AuditScope::DeclaredOnly => {
            println!();
            println!(
                "  {}",
                "No lockfile found — only declared dependencies were checked. Transitive \
                 dependencies are NOT covered by this result."
                    .yellow()
            );
        }
    }

    if !report.details_complete {
        println!(
            "  {}",
            "Some advisory records couldn't be fetched; those entries show an ID only, \
             without a severity."
                .yellow()
        );
    }
}

fn render_source_note(report: &AuditReport) {
    let ecosystem = report.ecosystem.unwrap_or("unknown");
    println!();
    println!(
        "  {}",
        format!(
            "Source: osv.dev (Open Source Vulnerabilities), {ecosystem} ecosystem only. \
             Full details at osv.dev/vulnerability/<id>."
        )
        .dimmed()
    );
    println!();
}
