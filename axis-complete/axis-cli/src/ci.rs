use colored::*;
use crate::types::Report;

pub fn run_ci(report: &Report, threshold: u32, json: bool) {
    if json {
        let out = serde_json::json!({
            "url": report.url,
            "score": report.score,
            "tier": report.tier,
            "threshold": threshold,
            "passed": report.score >= threshold,
            "errors": report.summary.errors,
            "warnings": report.summary.warnings,
            "pillars": {
                "perceivable": report.pillars.perceivable,
                "operable": report.pillars.operable,
                "understandable": report.pillars.understandable,
                "robust": report.pillars.robust,
            },
            "issues": report.issues.iter().map(|i| serde_json::json!({
                "severity": i.severity,
                "rule": i.rule,
                "pillar": i.wcag_pillar,
                "message": i.message,
                "location": i.location,
                "fix": i.fix,
                "score_impact": i.score_impact,
            })).collect::<Vec<_>>(),
        });
        println!("{}", serde_json::to_string_pretty(&out).unwrap());
    } else {
        println!();
        println!("{}", "════════════════════════════════════════════════".bold());
        println!("{}", "   AXIS  CI Check".bold());
        println!("{}", "════════════════════════════════════════════════".bold());
        println!("  URL:       {}", report.url.cyan());
        println!(
            "  Score:     {}/100  (threshold: {})",
            report.score, threshold
        );
        println!("  Tier:      {}", report.tier);
        println!();
        println!("{}", "  WCAG 2.2 Pillar Scores".bold());
        println!("{}", "  ──────────────────────────────────────────────".dimmed());
        println!("  Perceivable:    {}/25", report.pillars.perceivable);
        println!("  Operable:       {}/25", report.pillars.operable);
        println!("  Understandable: {}/25", report.pillars.understandable);
        println!("  Robust:         {}/25", report.pillars.robust);
        println!();

        if !report.issues.is_empty() {
            println!("{}", "  Issues".bold());
            println!("{}", "  ──────────────────────────────────────────────".dimmed());
            for issue in &report.issues {
                println!(
                    "  [{}] {} — {}",
                    issue.severity, issue.rule, issue.message
                );
                println!("         Location: {}", issue.location);
                println!(
                    "         Fix: {} (+{} score)",
                    issue.fix, issue.score_impact
                );
            }
            println!();
        }

        if ci_passes(report.score, threshold) {
            println!(
                "  {}  Score {} meets threshold {}",
                "PASS".green().bold(),
                report.score,
                threshold
            );
        } else {
            eprintln!(
                "  {}  Score {} is below threshold {}",
                "FAIL".red().bold(),
                report.score,
                threshold
            );
        }
        println!("{}", "════════════════════════════════════════════════".bold());
        println!();
    }

    if !ci_passes(report.score, threshold) {
        std::process::exit(1);
    }
}

pub fn ci_passes(score: u32, threshold: u32) -> bool {
    score >= threshold
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ci_passes_at_threshold() {
        assert!(ci_passes(80, 80));
    }

    #[test]
    fn test_ci_fails_below_threshold() {
        assert!(!ci_passes(79, 80));
        assert!(!ci_passes(0, 80));
    }

    #[test]
    fn test_ci_passes_above_threshold() {
        assert!(ci_passes(81, 80));
        assert!(ci_passes(100, 80));
    }
}
