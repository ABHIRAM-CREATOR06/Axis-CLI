use scraper::{Html, Selector};
use crate::types::{Issue, PillarScores};

pub fn run_checks(html: &str, source: &str) -> Vec<Issue> {
    let document = Html::parse_document(html);
    let mut issues = Vec::new();

    check_images(&document, source, &mut issues);
    check_lang(&document, source, &mut issues);
    check_title(&document, source, &mut issues);
    check_headings(&document, source, &mut issues);
    check_form_labels(&document, source, &mut issues);
    check_autocomplete(&document, source, &mut issues);
    check_target_size(&document, source, &mut issues);
    check_button_names(&document, source, &mut issues);
    check_link_text(&document, source, &mut issues);
    check_skip_link(&document, source, &mut issues);

    issues
}

fn check_images(document: &Html, source: &str, issues: &mut Vec<Issue>) {
    let sel = Selector::parse("img").unwrap();
    let mut missing = 0;
    let mut first_loc = String::new();

    for (i, img) in document.select(&sel).enumerate() {
        if img.value().attr("alt").is_none() {
            missing += 1;
            if first_loc.is_empty() {
                let src = img.value().attr("src").unwrap_or("unknown");
                first_loc = format!("img[{}] src=\"{}\"", i + 1, src);
            }
        }
    }

    if missing > 0 {
        issues.push(Issue {
            severity: "Error".to_string(),
            rule: "1.1.1 Non-text Content".to_string(),
            wcag_pillar: "Perceivable".to_string(),
            message: format!("{} image(s) missing alt attribute", missing),
            location: format!("{} → {}", source, first_loc),
            fix: "Add alt=\"description\" to all img elements".to_string(),
            score_impact: 10,
            persona_hint:
                "Screen reader users hear nothing when encountering these images \
                 — the content is completely invisible to them."
                    .to_string(),
        });
    }
}

fn check_lang(document: &Html, source: &str, issues: &mut Vec<Issue>) {
    let sel = Selector::parse("html").unwrap();
    if let Some(html_el) = document.select(&sel).next() {
        if html_el.value().attr("lang").is_none() {
            issues.push(Issue {
                severity: "Error".to_string(),
                rule: "3.1.1 Language of Page".to_string(),
                wcag_pillar: "Understandable".to_string(),
                message: "HTML element is missing a lang attribute".to_string(),
                location: format!("{} → <html>", source),
                fix: "Add lang=\"en\" to the <html> element".to_string(),
                score_impact: 10,
                persona_hint:
                    "Screen readers cannot switch to the correct pronunciation engine \
                     — text sounds garbled to non-English speakers using assistive technology."
                        .to_string(),
            });
        }
    }
}

fn check_title(document: &Html, source: &str, issues: &mut Vec<Issue>) {
    let sel = Selector::parse("title").unwrap();
    let title = document
        .select(&sel)
        .next()
        .map(|t| t.text().collect::<String>())
        .unwrap_or_default();

    if title.trim().is_empty() {
        issues.push(Issue {
            severity: "Error".to_string(),
            rule: "2.4.2 Page Titled".to_string(),
            wcag_pillar: "Operable".to_string(),
            message: "Page is missing a descriptive title".to_string(),
            location: format!("{} → <head><title>", source),
            fix: "Add a meaningful <title> inside <head>".to_string(),
            score_impact: 10,
            persona_hint:
                "Keyboard and screen reader users rely on the page title to understand \
                 where they are — without it, browser tabs are indistinguishable."
                    .to_string(),
        });
    }
}

fn check_headings(document: &Html, source: &str, issues: &mut Vec<Issue>) {
    let sel = Selector::parse("h1").unwrap();
    let h1_count = document.select(&sel).count();

    if h1_count == 0 {
        issues.push(Issue {
            severity: "Warning".to_string(),
            rule: "1.3.1 Info and Relationships".to_string(),
            wcag_pillar: "Perceivable".to_string(),
            message: "Page has no h1 heading".to_string(),
            location: format!("{} → <body>", source),
            fix: "Add a single h1 as the main page heading".to_string(),
            score_impact: 4,
            persona_hint:
                "Screen reader users navigate pages by headings — without an h1 \
                 there is no entry point to understand the page structure."
                    .to_string(),
        });
    } else if h1_count > 1 {
        issues.push(Issue {
            severity: "Warning".to_string(),
            rule: "1.3.1 Info and Relationships".to_string(),
            wcag_pillar: "Perceivable".to_string(),
            message: format!("Page has {} h1 headings, expected 1", h1_count),
            location: format!("{} → multiple h1 elements", source),
            fix: "Use only one h1 per page".to_string(),
            score_impact: 3,
            persona_hint:
                "Multiple h1 elements confuse screen reader users about which \
                 heading is the primary topic of the page."
                    .to_string(),
        });
    }
}

fn check_form_labels(document: &Html, source: &str, issues: &mut Vec<Issue>) {
    let input_sel = Selector::parse("input:not([type='hidden'])").unwrap();
    let label_sel = Selector::parse("label").unwrap();

    let labels: Vec<_> = document
        .select(&label_sel)
        .filter_map(|l| l.value().attr("for"))
        .collect();

    let mut unlabelled: u32 = 0;
    let mut first_name = String::new();

    for input in document.select(&input_sel) {
        let id = input.value().attr("id");
        let aria = input.value().attr("aria-label");
        let aria_by = input.value().attr("aria-labelledby");

        if aria.is_none()
            && aria_by.is_none()
            && !id.map(|i| labels.contains(&i)).unwrap_or(false)
        {
            unlabelled += 1;
            if first_name.is_empty() {
                first_name = input
                    .value()
                    .attr("name")
                    .or(input.value().attr("id"))
                    .or(input.value().attr("type"))
                    .unwrap_or("unknown")
                    .to_string();
            }
        }
    }

    if unlabelled > 0 {
        issues.push(Issue {
            severity: "Error".to_string(),
            rule: "1.3.5 Identify Input Purpose".to_string(),
            wcag_pillar: "Perceivable".to_string(),
            message: format!("{} input(s) missing associated label", unlabelled),
            location: format!(
                "{} → input[name=\"{}\"] (+{} more)",
                source,
                first_name,
                unlabelled.saturating_sub(1)
            ),
            fix: "Add <label for=\"id\"> or aria-label to each input".to_string(),
            score_impact: 10,
            persona_hint:
                "Screen reader users cannot determine what information to enter \
                 — unlabelled fields are announced only as \"edit text\"."
                    .to_string(),
        });
    }
}

fn check_autocomplete(document: &Html, source: &str, issues: &mut Vec<Issue>) {
    let sel = Selector::parse("input:not([type='hidden'])").unwrap();
    let personal = ["name", "email", "tel", "address", "phone"];
    let mut missing = 0;
    let mut first_field = String::new();

    for input in document.select(&sel) {
        let name = input.value().attr("name").unwrap_or("");
        let input_type = input.value().attr("type").unwrap_or("");
        let autocomplete = input.value().attr("autocomplete");

        if autocomplete.is_none()
            && (personal.iter().any(|p| name.contains(p))
                || input_type == "email"
                || input_type == "tel")
        {
            missing += 1;
            if first_field.is_empty() {
                first_field = name.to_string();
            }
        }
    }

    if missing > 0 {
        issues.push(Issue {
            severity: "Error".to_string(),
            rule: "3.3.7 Redundant Entry".to_string(),
            wcag_pillar: "Understandable".to_string(),
            message: format!(
                "{} personal data field(s) missing autocomplete",
                missing
            ),
            location: format!("{} → input[name=\"{}\"]", source, first_field),
            fix: "Add autocomplete=\"email\" / \"name\" / \"tel\" etc.".to_string(),
            score_impact: 10,
            persona_hint:
                "Users with cognitive disabilities or motor impairments must retype \
                 personal data every time — autocomplete removes this burden."
                    .to_string(),
        });
    }
}

fn check_target_size(document: &Html, source: &str, issues: &mut Vec<Issue>) {
    let sel = Selector::parse("button, a").unwrap();
    let mut small = 0;
    let mut first_loc = String::new();

    for (i, el) in document.select(&sel).enumerate() {
        if let Some(style) = el.value().attr("style") {
            if style.contains("width: 1") || style.contains("height: 1") {
                small += 1;
                if first_loc.is_empty() {
                    first_loc = format!("{}[{}]", el.value().name(), i + 1);
                }
            }
        }
    }

    if small > 0 {
        issues.push(Issue {
            severity: "Warning".to_string(),
            rule: "2.5.8 Target Size Minimum".to_string(),
            wcag_pillar: "Operable".to_string(),
            message: format!(
                "{} element(s) may have click target below 24x24px",
                small
            ),
            location: format!("{} → {}", source, first_loc),
            fix: "Set min-width: 24px; min-height: 24px on interactive elements"
                .to_string(),
            score_impact: 3,
            persona_hint:
                "Users with motor disabilities or tremors struggle to tap or click \
                 small targets — especially on mobile devices."
                    .to_string(),
        });
    }
}

fn check_button_names(document: &Html, source: &str, issues: &mut Vec<Issue>) {
    let sel = Selector::parse("button").unwrap();
    let mut nameless = 0;
    let mut first_loc = String::new();

    for (i, button) in document.select(&sel).enumerate() {
        let text = button.text().collect::<String>();
        let aria = button.value().attr("aria-label");
        let aria_by = button.value().attr("aria-labelledby");

        if text.trim().is_empty() && aria.is_none() && aria_by.is_none() {
            nameless += 1;
            if first_loc.is_empty() {
                first_loc = format!("button[{}]", i + 1);
            }
        }
    }

    if nameless > 0 {
        issues.push(Issue {
            severity: "Error".to_string(),
            rule: "4.1.2 Name, Role, Value".to_string(),
            wcag_pillar: "Robust".to_string(),
            message: format!("{} button(s) have no accessible name", nameless),
            location: format!("{} → {}", source, first_loc),
            fix: "Add visible text or aria-label to the button".to_string(),
            score_impact: 10,
            persona_hint:
                "Screen readers announce these buttons as simply \"button\" with no \
                 context — users cannot know what action they trigger."
                    .to_string(),
        });
    }
}

fn check_link_text(document: &Html, source: &str, issues: &mut Vec<Issue>) {
    let sel = Selector::parse("a").unwrap();
    let vague = ["click here", "read more", "here", "more", "link"];
    let mut bad_links = 0;
    let mut first_text = String::new();

    for link in document.select(&sel) {
        let text = link.text().collect::<String>().to_lowercase();
        let aria = link.value().attr("aria-label");

        if aria.is_none() && vague.iter().any(|v| text.trim() == *v) {
            bad_links += 1;
            if first_text.is_empty() {
                first_text = text.trim().to_string();
            }
        }
    }

    if bad_links > 0 {
        issues.push(Issue {
            severity: "Warning".to_string(),
            rule: "2.4.6 Headings and Labels".to_string(),
            wcag_pillar: "Operable".to_string(),
            message: format!(
                "{} link(s) have vague text like \"{}\"",
                bad_links, first_text
            ),
            location: format!("{} → <a> elements", source),
            fix: "Use descriptive link text that makes sense out of context".to_string(),
            score_impact: 3,
            persona_hint:
                "Screen reader users often navigate by listing all links on a page \
                 — \"click here\" repeated multiple times provides zero context."
                    .to_string(),
        });
    }
}

fn check_skip_link(document: &Html, source: &str, issues: &mut Vec<Issue>) {
    let sel = Selector::parse("a").unwrap();
    let has_skip = document.select(&sel).any(|a| {
        let href = a.value().attr("href").unwrap_or("");
        href == "#main" || href == "#content" || href == "#main-content"
    });

    if !has_skip {
        issues.push(Issue {
            severity: "Warning".to_string(),
            rule: "2.4.1 Bypass Blocks".to_string(),
            wcag_pillar: "Operable".to_string(),
            message: "No skip navigation link found".to_string(),
            location: format!("{} → <body> start", source),
            fix: "Add <a href=\"#main\">Skip to main content</a> as first element"
                .to_string(),
            score_impact: 3,
            persona_hint:
                "Keyboard users must tab through every navigation item on every \
                 page load — a skip link lets them jump straight to content."
                    .to_string(),
        });
    }
}

pub fn calculate_score(issues: &[Issue]) -> u32 {
    let penalty: u32 = issues
        .iter()
        .map(|i| {
            if i.severity == "Error" {
                i.score_impact
            } else {
                i.score_impact / 2
            }
        })
        .sum();
    100u32.saturating_sub(penalty)
}

pub fn calculate_pillars(issues: &[Issue]) -> PillarScores {
    let pillar_penalty = |pillar: &str| -> u32 {
        issues
            .iter()
            .filter(|i| i.wcag_pillar == pillar)
            .map(|i| {
                if i.severity == "Error" {
                    i.score_impact
                } else {
                    i.score_impact / 2
                }
            })
            .sum()
    };

    PillarScores {
        perceivable:    25u32.saturating_sub(pillar_penalty("Perceivable")),
        operable:       25u32.saturating_sub(pillar_penalty("Operable")),
        understandable: 25u32.saturating_sub(pillar_penalty("Understandable")),
        robust:         25u32.saturating_sub(pillar_penalty("Robust")),
    }
}

pub fn get_tier(score: u32) -> &'static str {
    match score {
        95..=100 => "Fully Compliant",
        80..=94  => "Mostly Compliant",
        60..=79  => "Partially Compliant",
        _        => "Not Compliant",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── helpers ──────────────────────────────────────────────────────────────

    /// Fully-compliant minimal HTML page (passes all 10 checks).
    fn perfect_html() -> &'static str {
        r##"<!DOCTYPE html>
<html lang="en">
<head><title>My Site</title></head>
<body>
  <a href="#main">Skip to main content</a>
  <main id="main">
    <h1>Welcome</h1>
    <img src="hero.jpg" alt="Hero image">
    <a href="/docs">Read the documentation</a>
    <form>
      <label for="email">Email</label>
      <input id="email" name="email" type="email" autocomplete="email">
      <button>Submit</button>
    </form>
  </main>
</body>
</html>"##
    }

    fn run(html: &str) -> Vec<Issue> {
        run_checks(html, "test://source")
    }

    fn has_rule(issues: &[Issue], rule_prefix: &str) -> bool {
        issues.iter().any(|i| i.rule.starts_with(rule_prefix))
    }

    fn make_issue(severity: &str, pillar: &str, impact: u32) -> Issue {
        Issue {
            severity:     severity.to_string(),
            rule:         "X.X.X Test".to_string(),
            wcag_pillar:  pillar.to_string(),
            message:      "test".to_string(),
            location:     "test".to_string(),
            fix:          "test".to_string(),
            score_impact: impact,
            persona_hint: "test".to_string(),
        }
    }

    // ── perfect HTML ─────────────────────────────────────────────────────────

    #[test]
    fn test_perfect_html_no_issues() {
        let issues = run(perfect_html());
        assert!(
            issues.is_empty(),
            "expected no issues but got: {:?}",
            issues.iter().map(|i| &i.rule).collect::<Vec<_>>()
        );
    }

    // ── 1.1.1 Images ─────────────────────────────────────────────────────────

    #[test]
    fn test_images_missing_alt() {
        let html = r##"<html lang="en"><head><title>T</title></head><body><h1>H</h1><a href="#main">Skip</a><img src="hero.jpg"></body></html>"##;
        let issues = run(html);
        assert!(has_rule(&issues, "1.1.1"), "expected 1.1.1 issue");
        let issue = issues.iter().find(|i| i.rule.starts_with("1.1.1")).unwrap();
        assert_eq!(issue.severity, "Error");
        assert!(issue.message.contains("1 image"));
    }

    #[test]
    fn test_images_with_alt_ok() {
        let html = r##"<html lang="en"><head><title>T</title></head><body><h1>H</h1><a href="#main">Skip</a><img src="hero.jpg" alt="desc"></body></html>"##;
        assert!(!has_rule(&run(html), "1.1.1"), "unexpected 1.1.1 issue");
    }

    #[test]
    fn test_multiple_images_missing_alt_count() {
        let html = r##"<html lang="en"><head><title>T</title></head><body><h1>H</h1><a href="#main">Skip</a><img src="a.jpg"><img src="b.jpg"></body></html>"##;
        let issues = run(html);
        let issue = issues.iter().find(|i| i.rule.starts_with("1.1.1")).unwrap();
        assert!(issue.message.contains("2 image"), "message: {}", issue.message);
    }

    // ── 3.1.1 lang ───────────────────────────────────────────────────────────

    #[test]
    fn test_missing_lang() {
        let html = r##"<html><head><title>T</title></head><body><h1>H</h1><a href="#main">Skip</a></body></html>"##;
        assert!(has_rule(&run(html), "3.1.1"), "expected 3.1.1 issue");
    }

    #[test]
    fn test_lang_present_ok() {
        let html = r##"<html lang="en"><head><title>T</title></head><body><h1>H</h1><a href="#main">Skip</a></body></html>"##;
        assert!(!has_rule(&run(html), "3.1.1"), "unexpected 3.1.1 issue");
    }

    // ── 2.4.2 title ──────────────────────────────────────────────────────────

    #[test]
    fn test_missing_title() {
        let html = r##"<html lang="en"><head></head><body><h1>H</h1><a href="#main">Skip</a></body></html>"##;
        assert!(has_rule(&run(html), "2.4.2"), "expected 2.4.2 issue");
    }

    #[test]
    fn test_title_present_ok() {
        let html = r##"<html lang="en"><head><title>My Page</title></head><body><h1>H</h1><a href="#main">Skip</a></body></html>"##;
        assert!(!has_rule(&run(html), "2.4.2"), "unexpected 2.4.2 issue");
    }

    // ── 1.3.1 headings ───────────────────────────────────────────────────────

    #[test]
    fn test_no_h1() {
        let html = r##"<html lang="en"><head><title>T</title></head><body><a href="#main">Skip</a></body></html>"##;
        let issues = run(html);
        assert!(has_rule(&issues, "1.3.1"), "expected 1.3.1 issue");
        let issue = issues.iter().find(|i| i.rule.starts_with("1.3.1")).unwrap();
        assert!(issue.message.contains("no h1"), "message: {}", issue.message);
    }

    #[test]
    fn test_multiple_h1() {
        let html = r##"<html lang="en"><head><title>T</title></head><body><h1>A</h1><h1>B</h1><a href="#main">Skip</a></body></html>"##;
        let issues = run(html);
        assert!(has_rule(&issues, "1.3.1"), "expected 1.3.1 issue");
        let issue = issues.iter().find(|i| i.rule.starts_with("1.3.1")).unwrap();
        assert!(issue.message.contains('2'), "message should mention count 2: {}", issue.message);
    }

    #[test]
    fn test_single_h1_ok() {
        let html = r##"<html lang="en"><head><title>T</title></head><body><h1>Main</h1><a href="#main">Skip</a></body></html>"##;
        assert!(!has_rule(&run(html), "1.3.1"), "unexpected 1.3.1 issue");
    }

    // ── 1.3.5 form labels ────────────────────────────────────────────────────

    #[test]
    fn test_unlabelled_input() {
        let html = r##"<html lang="en"><head><title>T</title></head><body><h1>H</h1><a href="#main">Skip</a><form><input id="x" name="x"></form></body></html>"##;
        let issues = run(html);
        assert!(has_rule(&issues, "1.3.5"), "expected 1.3.5 issue");
        assert_eq!(issues.iter().find(|i| i.rule.starts_with("1.3.5")).unwrap().severity, "Error");
    }

    #[test]
    fn test_labelled_input_ok() {
        let html = r##"<html lang="en"><head><title>T</title></head><body><h1>H</h1><a href="#main">Skip</a><form><label for="x">Name</label><input id="x" name="x"></form></body></html>"##;
        assert!(!has_rule(&run(html), "1.3.5"), "unexpected 1.3.5 issue");
    }

    #[test]
    fn test_aria_label_ok() {
        let html = r##"<html lang="en"><head><title>T</title></head><body><h1>H</h1><a href="#main">Skip</a><form><input aria-label="Email" name="x"></form></body></html>"##;
        assert!(!has_rule(&run(html), "1.3.5"), "unexpected 1.3.5 with aria-label");
    }

    // ── 4.1.2 button names ───────────────────────────────────────────────────

    #[test]
    fn test_nameless_button() {
        let html = r##"<html lang="en"><head><title>T</title></head><body><h1>H</h1><a href="#main">Skip</a><button></button></body></html>"##;
        let issues = run(html);
        assert!(has_rule(&issues, "4.1.2"), "expected 4.1.2 issue");
        assert_eq!(issues.iter().find(|i| i.rule.starts_with("4.1.2")).unwrap().severity, "Error");
    }

    #[test]
    fn test_named_button_ok() {
        let html = r##"<html lang="en"><head><title>T</title></head><body><h1>H</h1><a href="#main">Skip</a><button>Submit</button></body></html>"##;
        assert!(!has_rule(&run(html), "4.1.2"), "unexpected 4.1.2 issue");
    }

    // ── 2.4.6 link text ──────────────────────────────────────────────────────

    #[test]
    fn test_vague_link() {
        let html = r##"<html lang="en"><head><title>T</title></head><body><h1>H</h1><a href="#main">Skip</a><a href="/docs">click here</a></body></html>"##;
        assert!(has_rule(&run(html), "2.4.6"), "expected 2.4.6 issue");
    }

    #[test]
    fn test_descriptive_link_ok() {
        let html = r##"<html lang="en"><head><title>T</title></head><body><h1>H</h1><a href="#main">Skip to content</a><a href="/docs">Read the documentation</a></body></html>"##;
        assert!(!has_rule(&run(html), "2.4.6"), "unexpected 2.4.6 issue");
    }

    // ── 2.4.1 skip link ──────────────────────────────────────────────────────

    #[test]
    fn test_no_skip_link() {
        let html = r##"<html lang="en"><head><title>T</title></head><body><h1>H</h1></body></html>"##;
        assert!(has_rule(&run(html), "2.4.1"), "expected 2.4.1 issue");
    }

    #[test]
    fn test_skip_link_ok() {
        let html = r##"<html lang="en"><head><title>T</title></head><body><a href="#main">Skip</a><h1>H</h1></body></html>"##;
        assert!(!has_rule(&run(html), "2.4.1"), "unexpected 2.4.1 issue");
    }

    // ── calculate_score ──────────────────────────────────────────────────────

    #[test]
    fn test_calculate_score_no_issues() {
        assert_eq!(calculate_score(&[]), 100);
    }

    #[test]
    fn test_calculate_score_errors_only() {
        // Two Errors with impact 10 each → penalty 20 → score 80
        let issues = vec![
            make_issue("Error", "Perceivable", 10),
            make_issue("Error", "Operable",    10),
        ];
        assert_eq!(calculate_score(&issues), 80);
    }

    #[test]
    fn test_calculate_score_warnings_halved() {
        // Warning with impact 4 → penalty 2 → score 98
        let issues = vec![make_issue("Warning", "Operable", 4)];
        assert_eq!(calculate_score(&issues), 98);
    }

    #[test]
    fn test_calculate_score_saturates_at_zero() {
        // Many high-impact errors must not underflow
        let issues: Vec<Issue> = (0..20).map(|_| make_issue("Error", "Perceivable", 10)).collect();
        assert_eq!(calculate_score(&issues), 0);
    }

    // ── get_tier ─────────────────────────────────────────────────────────────

    #[test]
    fn test_get_tier_boundaries() {
        assert_eq!(get_tier(100), "Fully Compliant");
        assert_eq!(get_tier(95),  "Fully Compliant");
        assert_eq!(get_tier(94),  "Mostly Compliant");
        assert_eq!(get_tier(80),  "Mostly Compliant");
        assert_eq!(get_tier(79),  "Partially Compliant");
        assert_eq!(get_tier(60),  "Partially Compliant");
        assert_eq!(get_tier(59),  "Not Compliant");
        assert_eq!(get_tier(0),   "Not Compliant");
    }

    // ── calculate_pillars ────────────────────────────────────────────────────

    #[test]
    fn test_calculate_pillars_no_issues() {
        let p = calculate_pillars(&[]);
        assert_eq!(p.perceivable, 25);
        assert_eq!(p.operable, 25);
        assert_eq!(p.understandable, 25);
        assert_eq!(p.robust, 25);
    }

    #[test]
    fn test_calculate_pillars_single_perceivable_error() {
        let issues = vec![make_issue("Error", "Perceivable", 10)];
        let p = calculate_pillars(&issues);
        assert_eq!(p.perceivable,    15); // 25 - 10
        assert_eq!(p.operable,       25);
        assert_eq!(p.understandable, 25);
        assert_eq!(p.robust,         25);
    }

    #[test]
    fn test_calculate_pillars_saturates_at_zero() {
        let issues: Vec<Issue> = (0..5).map(|_| make_issue("Error", "Robust", 10)).collect();
        let p = calculate_pillars(&issues);
        assert_eq!(p.robust,    0);
        assert_eq!(p.operable, 25); // other pillars unaffected
    }
}
