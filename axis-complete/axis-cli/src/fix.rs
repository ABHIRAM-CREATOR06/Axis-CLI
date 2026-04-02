use colored::*;
use crate::types::Report;

pub fn run_fix_preview(report: &Report) {
    let score_before = report.score;
    let fixable: Vec<_> = report.issues.iter().collect();
    let gain: u32 = compute_fix_gain(&fixable);
    let score_after = (score_before + gain).min(100);

    println!();
    println!("{}", "════════════════════════════════════════════════".bold());
    println!("{}", "   AXIS  Fix Preview".bold());
    println!("{}", "════════════════════════════════════════════════".bold());
    println!("  URL: {}", report.url.cyan());
    println!();
    println!(
        "  Score before:  {}",
        format!("{}/100", score_before).red().bold()
    );
    println!(
        "  Score after:   {} {}",
        format!("{}/100", score_after).green().bold(),
        format!("(+{})", gain).green().bold()
    );
    println!();

    // Pillar improvements
    println!("{}", "  Pillar improvement after fixes".bold());
    println!("{}", "  ──────────────────────────────────────────────".dimmed());
    println!(
        "  Perceivable:    {}/25  →  would improve",
        report.pillars.perceivable
    );
    println!(
        "  Operable:       {}/25  →  would improve",
        report.pillars.operable
    );
    println!(
        "  Understandable: {}/25  →  would improve",
        report.pillars.understandable
    );
    println!(
        "  Robust:         {}/25  →  would improve",
        report.pillars.robust
    );
    println!();

    println!(
        "  {} issue(s) fixable",
        fixable.len().to_string().green().bold()
    );
    println!("{}", "  ──────────────────────────────────────────────".dimmed());

    for issue in &fixable {
        let gain_for_issue = if issue.severity == "Error" {
            issue.score_impact
        } else {
            issue.score_impact / 2
        };

        println!();
        println!(
            "  {} {} {}",
            "✓".green().bold(),
            issue.rule.bold(),
            format!("(+{} score)", gain_for_issue).green()
        );
        println!("    {}", issue.message);
        println!(
            "    {}  {}",
            "Fix:".dimmed(),
            issue.fix
        );
        println!(
            "    {}  {}",
            "Location:".dimmed(),
            issue.location.yellow()
        );
        println!(
            "    {}  {}",
            "Impact:".magenta(),
            issue.persona_hint.italic()
        );
    }

    println!();
    println!("{}", "════════════════════════════════════════════════".bold());
    println!();
}

pub fn compute_fix_gain(issues: &[&crate::types::Issue]) -> u32 {
    issues
        .iter()
        .map(|i| {
            if i.severity == "Error" {
                i.score_impact
            } else {
                i.score_impact / 2
            }
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Issue;

    fn make_issue(severity: &str, impact: u32) -> Issue {
        Issue {
            severity: severity.to_string(),
            rule: "Test".to_string(),
            wcag_pillar: "Test".to_string(),
            message: "Test".to_string(),
            location: "Test".to_string(),
            fix: "Test".to_string(),
            score_impact: impact,
            persona_hint: "Test".to_string(),
        }
    }

    #[test]
    fn test_fix_gain_no_issues() {
        assert_eq!(compute_fix_gain(&[]), 0);
    }

    #[test]
    fn test_fix_gain_errors() {
        let i1 = make_issue("Error", 10);
        let i2 = make_issue("Error", 20);
        assert_eq!(compute_fix_gain(&[&i1, &i2]), 30);
    }

    #[test]
    fn test_fix_gain_warnings_halved() {
        let i1 = make_issue("Warning", 4);
        let i2 = make_issue("Warning", 10);
        assert_eq!(compute_fix_gain(&[&i1, &i2]), 7);
    }

    #[test]
    fn test_fix_gain_mixed() {
        let e1 = make_issue("Error", 10);
        let w1 = make_issue("Warning", 4);
        assert_eq!(compute_fix_gain(&[&e1, &w1]), 12);
    }
}
