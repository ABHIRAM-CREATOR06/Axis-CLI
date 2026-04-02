use colored::*;
use walkdir::WalkDir;
use crate::checker::{calculate_pillars, calculate_score, get_tier, run_checks};
use crate::output::print_text;
use crate::renderer::render_file;
use crate::types::{Report, Summary};

pub fn scan_local(path: &str, format: &str, output: Option<String>) {
    println!();
    println!(
        "  {} {}",
        "Scanning local project:".dimmed(),
        path.cyan().bold()
    );
    println!();

    let html_files: Vec<_> = WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "html" || ext == "htm")
                .unwrap_or(false)
        })
        .collect();

    if html_files.is_empty() {
        println!(
            "  {} No HTML files found in {}",
            "!".yellow().bold(),
            path
        );
        return;
    }

    println!(
        "  Found {} HTML file(s)\n",
        html_files.len().to_string().green().bold()
    );

    let mut all_reports: Vec<Report> = Vec::new();

    for entry in &html_files {
        let file_path = entry.path().to_string_lossy().to_string();

        let rendered = match render_file(&file_path) {
            Ok(r) => r,
            Err(e) => {
                println!(
                    "  {} Could not read {}: {}",
                    "!".red(),
                    file_path,
                    e
                );
                continue;
            }
        };

        let issues = run_checks(&rendered.html, &file_path);
        let score = calculate_score(&issues);
        let tier = get_tier(score).to_string();
        let errors = issues.iter().filter(|i| i.severity == "Error").count() as u32;
        let warnings = issues.iter().filter(|i| i.severity == "Warning").count() as u32;
        let total_checks = 10u32;
        let passed = total_checks.saturating_sub(errors + warnings);
        let pillars = calculate_pillars(&issues);

        let report = Report {
            url: file_path.clone(),
            score,
            tier,
            issues,
            summary: Summary { errors, warnings, passed },
            pillars,
            load_time_ms: 0,
            total_bytes: 0,
            request_count: 0,
        };

        // Per-file summary line
        let pass_icon = if score >= 80 {
            "✓".green()
        } else {
            "✗".red()
        };

        println!(
            "  {}  {}",
            pass_icon,
            file_path.bold()
        );
        println!(
            "     Score: {}  Tier: {}",
            match score {
                95..=100 => format!("{}/100", score).green().bold(),
                80..=94  => format!("{}/100", score).cyan().bold(),
                60..=79  => format!("{}/100", score).yellow().bold(),
                _        => format!("{}/100", score).red().bold(),
            },
            report.tier
        );
        println!(
            "     Perceivable: {}/25  Operable: {}/25  \
             Understandable: {}/25  Robust: {}/25",
            report.pillars.perceivable,
            report.pillars.operable,
            report.pillars.understandable,
            report.pillars.robust,
        );
        println!(
            "     {} errors  {} warnings",
            errors.to_string().red(),
            warnings.to_string().yellow()
        );

        if !report.issues.is_empty() {
            for issue in &report.issues {
                println!(
                    "       {} {} (+{} if fixed)",
                    "→".dimmed(),
                    issue.rule,
                    issue.score_impact.to_string().green()
                );
                println!(
                    "         {} {}",
                    "Location:".dimmed(),
                    issue.location.yellow()
                );
            }
        }

        println!();
        all_reports.push(report);
    }

    // Project-wide summary
    let total = all_reports.len();
    let avg = if total > 0 {
        all_reports.iter().map(|r| r.score).sum::<u32>() / total as u32
    } else {
        0
    };
    let passing = all_reports.iter().filter(|r| r.score >= 80).count();
    let total_errors: u32 = all_reports.iter().map(|r| r.summary.errors).sum();
    let total_warnings: u32 = all_reports.iter().map(|r| r.summary.warnings).sum();

    let avg_perceivable = all_reports.iter().map(|r| r.pillars.perceivable).sum::<u32>()
        / total.max(1) as u32;
    let avg_operable = all_reports.iter().map(|r| r.pillars.operable).sum::<u32>()
        / total.max(1) as u32;
    let avg_understandable = all_reports
        .iter()
        .map(|r| r.pillars.understandable)
        .sum::<u32>()
        / total.max(1) as u32;
    let avg_robust = all_reports.iter().map(|r| r.pillars.robust).sum::<u32>()
        / total.max(1) as u32;

    println!("{}", "════════════════════════════════════════════════".bold());
    println!("{}", "   AXIS  Project Summary".bold());
    println!("{}", "════════════════════════════════════════════════".bold());
    println!("  Files scanned:  {}", total);
    println!(
        "  Average score:  {}",
        match avg {
            80..=100 => format!("{}/100", avg).green().bold(),
            60..=79  => format!("{}/100", avg).yellow().bold(),
            _        => format!("{}/100", avg).red().bold(),
        }
    );
    println!(
        "  Passing files:  {}/{}",
        passing.to_string().green().bold(),
        total
    );
    println!();
    println!("{}", "  Average WCAG Pillar Scores".bold());
    println!("{}", "  ──────────────────────────────────────────────".dimmed());
    println!("  Perceivable:    {}/25", avg_perceivable);
    println!("  Operable:       {}/25", avg_operable);
    println!("  Understandable: {}/25", avg_understandable);
    println!("  Robust:         {}/25", avg_robust);
    println!();
    println!(
        "  Total errors:   {}",
        total_errors.to_string().red().bold()
    );
    println!(
        "  Total warnings: {}",
        total_warnings.to_string().yellow().bold()
    );
    println!("{}", "════════════════════════════════════════════════".bold());
    println!();

    if let Some(out_path) = output {
        let json = serde_json::json!({
            "project_path": path,
            "files_scanned": total,
            "average_score": avg,
            "passing_files": passing,
            "total_errors": total_errors,
            "total_warnings": total_warnings,
            "average_pillars": {
                "perceivable": avg_perceivable,
                "operable": avg_operable,
                "understandable": avg_understandable,
                "robust": avg_robust,
            },
            "files": all_reports.iter().map(|r| serde_json::json!({
                "file": r.url,
                "score": r.score,
                "tier": r.tier,
                "errors": r.summary.errors,
                "warnings": r.summary.warnings,
                "pillars": {
                    "perceivable": r.pillars.perceivable,
                    "operable": r.pillars.operable,
                    "understandable": r.pillars.understandable,
                    "robust": r.pillars.robust,
                },
                "issues": r.issues,
            })).collect::<Vec<_>>()
        });
        std::fs::write(&out_path, serde_json::to_string_pretty(&json).unwrap())
            .unwrap();
        println!("Project report saved to {}", out_path.green());
    }
}
