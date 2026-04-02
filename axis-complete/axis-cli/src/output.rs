use colored::*;
use crate::types::Report;

pub fn print_text(report: &Report, output: Option<String>) {
    println!();
    println!("{}", "════════════════════════════════════════════════".bold());
    println!("{}", "   AXIS  Web Accessibility Report".bold());
    println!("{}", "════════════════════════════════════════════════".bold());
    println!("  URL:       {}", report.url.cyan());
    println!("  Score:     {}", score_colored(report.score));
    println!("  Tier:      {}", tier_colored(&report.tier));

    if report.load_time_ms > 0 {
        println!("  Load time: {}ms", report.load_time_ms);
    }
    if report.total_bytes > 0 {
        println!("  Page size: {}kb", report.total_bytes / 1024);
    }
    if report.request_count > 0 {
        println!("  Requests:  {}", report.request_count);
    }

    println!();
    println!("{}", "  WCAG 2.2 Pillar Scores".bold());
    println!("{}", "  ──────────────────────────────────────────────".dimmed());
    print_pillar("  Perceivable    ", report.pillars.perceivable);
    print_pillar("  Operable       ", report.pillars.operable);
    print_pillar("  Understandable ", report.pillars.understandable);
    print_pillar("  Robust         ", report.pillars.robust);

    println!();
    println!("{}", "  Summary".bold());
    println!("{}", "  ──────────────────────────────────────────────".dimmed());
    println!(
        "  {}  errors      {}  warnings      {}  passed",
        report.summary.errors.to_string().red().bold(),
        report.summary.warnings.to_string().yellow().bold(),
        report.summary.passed.to_string().green().bold(),
    );

    if report.issues.is_empty() {
        println!();
        println!(
            "  {}  No issues found — fully accessible!",
            "✓".green().bold()
        );
    } else {
        println!();
        println!("{}", "  Issues".bold());
        println!("{}", "  ──────────────────────────────────────────────".dimmed());

        for issue in &report.issues {
            let severity_label = match issue.severity.as_str() {
                "Error"   => "✗ ERROR  ".red().bold(),
                "Warning" => "⚠ WARNING".yellow().bold(),
                _         => "ℹ INFO   ".blue().bold(),
            };

            println!();
            println!(
                "  {} {} {}",
                severity_label,
                issue.rule.bold(),
                format!("[{}]", issue.wcag_pillar).dimmed()
            );
            println!("           {}", issue.message);
            println!(
                "  {}   {}",
                "Location:".dimmed(),
                issue.location.yellow()
            );
            println!(
                "  {}        {} {}",
                "Fix:".dimmed(),
                issue.fix,
                format!("(+{} score)", issue.score_impact).green().bold()
            );
            println!();
            println!(
                "  {} {}",
                "Impact:".magenta().bold(),
                issue.persona_hint.italic()
            );
        }
    }

    println!();
    println!("{}", "════════════════════════════════════════════════".bold());
    println!();

    if let Some(path) = output {
        let content = build_plain_report(report);
        std::fs::write(&path, &content).unwrap();
        println!("  Report saved to {}", path.green());
        println!();
    }
}

fn print_pillar(label: &str, score: u32) {
    let bar = build_bar(score, 25, 24);
    let score_str = format!("{}/25", score);
    let colored_score = match score {
        20..=25 => score_str.green().bold(),
        15..=19 => score_str.yellow().bold(),
        _       => score_str.red().bold(),
    };
    println!("  {}  {}  {}", label, bar, colored_score);
}

fn build_bar(score: u32, max: u32, width: u32) -> String {
    let filled = (score * width / max) as usize;
    let empty = width as usize - filled;
    format!(
        "{}{}",
        "█".repeat(filled).green(),
        "░".repeat(empty).dimmed()
    )
}

fn score_colored(score: u32) -> colored::ColoredString {
    let s = format!("{}/100", score);
    match score {
        95..=100 => s.green().bold(),
        80..=94  => s.cyan().bold(),
        60..=79  => s.yellow().bold(),
        _        => s.red().bold(),
    }
}

fn tier_colored(tier: &str) -> colored::ColoredString {
    match tier {
        "Fully Compliant"    => tier.green().bold(),
        "Mostly Compliant"   => tier.cyan().bold(),
        "Partially Compliant"=> tier.yellow().bold(),
        _                    => tier.red().bold(),
    }
}

fn build_plain_report(report: &Report) -> String {
    let mut out = String::new();
    out.push_str("════════════════════════════════════════════════\n");
    out.push_str("   AXIS  Web Accessibility Report\n");
    out.push_str("════════════════════════════════════════════════\n");
    out.push_str(&format!("  URL:   {}\n", report.url));
    out.push_str(&format!("  Score: {}/100\n", report.score));
    out.push_str(&format!("  Tier:  {}\n", report.tier));
    if report.load_time_ms > 0 {
        out.push_str(&format!("  Load:  {}ms\n", report.load_time_ms));
    }
    out.push('\n');
    out.push_str("  WCAG 2.2 Pillar Scores\n");
    out.push_str("  ──────────────────────────────────────────────\n");
    out.push_str(&format!("  Perceivable:    {}/25\n", report.pillars.perceivable));
    out.push_str(&format!("  Operable:       {}/25\n", report.pillars.operable));
    out.push_str(&format!("  Understandable: {}/25\n", report.pillars.understandable));
    out.push_str(&format!("  Robust:         {}/25\n", report.pillars.robust));
    out.push('\n');
    out.push_str("  Summary\n");
    out.push_str("  ──────────────────────────────────────────────\n");
    out.push_str(&format!(
        "  {} errors    {} warnings    {} passed\n",
        report.summary.errors,
        report.summary.warnings,
        report.summary.passed
    ));
    out.push('\n');

    if report.issues.is_empty() {
        out.push_str("  No issues found — fully accessible!\n");
    } else {
        out.push_str("  Issues\n");
        out.push_str("  ──────────────────────────────────────────────\n");
        for issue in &report.issues {
            out.push('\n');
            out.push_str(&format!(
                "  [{}] {} [{}]\n",
                issue.severity, issue.rule, issue.wcag_pillar
            ));
            out.push_str(&format!("       {}\n", issue.message));
            out.push_str(&format!("  Location: {}\n", issue.location));
            out.push_str(&format!(
                "  Fix:      {} (+{} score)\n",
                issue.fix, issue.score_impact
            ));
            out.push('\n');
            out.push_str(&format!("  Impact: {}\n", issue.persona_hint));
        }
    }

    out.push_str("\n════════════════════════════════════════════════\n");
    out
}

pub fn print_json(report: &Report, output: Option<String>) {
    let json = serde_json::to_string_pretty(report).unwrap();
    println!("{}", json);
    if let Some(path) = output {
        std::fs::write(&path, &json).unwrap();
        eprintln!("Report saved to {}", path);
    }
}
