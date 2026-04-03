# AXIS — Command Reference

Complete reference for every command, flag, and option in AXIS v0.2.0.

---

## Global

```
axis [COMMAND] [OPTIONS]
```

| Flag | Description |
|---|---|
| `--help` | Print help for any command |
| `--version` | Print version (`axis --version`) |

---

## `axis check`

Check a single URL or local HTML file for accessibility issues.

```
axis check <TARGET> [OPTIONS]
```

### Arguments

| Argument | Description |
|---|---|
| `<TARGET>` | A URL (`https://example.com`) or a local file path (`./index.html`) |

### Options

| Flag | Default | Description |
|---|---|---|
| `--headless` | off | Render via headless Chrome (requires `axis-render.exe` next to `axis.exe`) |
| `--format <FORMAT>` | `text` | Output format: `text` or `json` |
| `--output <FILE>` | — | Save the report to a file |
| `--threshold <N>` | `0` | Exit with code `1` if score is below `N` |

### Examples

```bash
# Basic HTTP check
axis check https://example.com

# Full JS rendering via headless Chrome
axis check https://example.com --headless

# Local HTML file
axis check ./index.html

# Local file with headless Chrome
axis check ./index.html --headless

# JSON output printed to terminal
axis check https://example.com --format json

# Save text report to file
axis check https://example.com --output report.txt

# Save JSON report to file
axis check https://example.com --format json --output report.json

# Fail (exit code 1) if score drops below 80
axis check https://example.com --threshold 80

# Combine: headless + JSON + save + threshold
axis check https://example.com --headless --format json --output report.json --threshold 90
```

### Exit codes

| Code | Meaning |
|---|---|
| `0` | Success (score meets threshold, or no threshold set) |
| `1` | Score is below `--threshold`, or a fatal error occurred |

### Sample output — text

```
  ════════════════════════════════════════════════
     AXIS  Web Accessibility Report
  ════════════════════════════════════════════════
    URL:       https://example.com
    Score:     77/100
    Tier:      Partially Compliant
    Load time: 6241ms
    Page size: 142kb
    Requests:  47

    WCAG 2.2 Pillar Scores
    ──────────────────────────────────────────────
    Perceivable      ██████████████████████░░  23/25
    Operable         █████████████░░░░░░░░░░░  14/25
    Understandable   ██████████████░░░░░░░░░░  15/25
    Robust           ████████████████████████  25/25

    Summary
    ──────────────────────────────────────────────
    2  errors      2  warnings      6  passed

    Issues
    ──────────────────────────────────────────────

    ✗ ERROR   3.1.1 Language of Page [Understandable]
              HTML element is missing a lang attribute
    Location: https://example.com → <html>
    Fix:      Add lang="en" to the <html> element (+10 score)

    Impact: Screen readers cannot switch to the correct pronunciation engine.
  ════════════════════════════════════════════════
```

### Sample output — JSON

```json
{
  "url": "https://example.com",
  "score": 77,
  "tier": "Partially Compliant",
  "load_time_ms": 6241,
  "total_bytes": 145408,
  "request_count": 47,
  "pillars": {
    "perceivable": 23,
    "operable": 14,
    "understandable": 15,
    "robust": 25
  },
  "summary": {
    "errors": 2,
    "warnings": 2,
    "passed": 6
  },
  "issues": [
    {
      "severity": "Error",
      "rule": "3.1.1 Language of Page",
      "wcag_pillar": "Understandable",
      "message": "HTML element is missing a lang attribute",
      "location": "https://example.com → <html>",
      "fix": "Add lang=\"en\" to the <html> element",
      "score_impact": 10,
      "persona_hint": "Screen readers cannot switch to the correct pronunciation engine — text sounds garbled to non-English speakers using assistive technology."
    }
  ]
}
```

---

## `axis scan`

Scan every HTML file in a local project directory recursively.

```
axis scan <PATH> [OPTIONS]
```

### Arguments

| Argument | Description |
|---|---|
| `<PATH>` | Path to the project directory to scan |

### Options

| Flag | Default | Description |
|---|---|---|
| `--format <FORMAT>` | `text` | Output format: `text` or `json` |
| `--output <FILE>` | — | Save the project report to a file |

### Examples

```bash
# Scan a local project
axis scan ./my-website/

# Scan and save JSON report
axis scan ./my-website/ --output project-report.json

# Scan a specific subdirectory
axis scan ./src/views/
```

### What it does

- Walks the directory tree recursively
- Finds every `.html` and `.htm` file
- Runs all 10 WCAG checks on each file
- Prints a per-file summary followed by a project-wide aggregate

### Sample output

```
  Scanning local project: ./my-website/

  Found 4 HTML file(s)

  ✓  ./my-website/index.html
     Score: 95/100  Tier: Fully Compliant
     Perceivable: 25/25  Operable: 25/25  Understandable: 25/25  Robust: 20/25
     0 errors  1 warnings

  ✗  ./my-website/contact.html
     Score: 68/100  Tier: Partially Compliant
     Perceivable: 15/25  Operable: 24/25  Understandable: 25/25  Robust: 25/25
     2 errors  1 warnings
       → 1.1.1 Non-text Content (+10 if fixed)
         Location: img[3] src="hero.jpg"

  ════════════════════════════════════════════════
     AXIS  Project Summary
  ════════════════════════════════════════════════
    Files scanned:  4
    Average score:  81/100
    Passing files:  3/4

    Average WCAG Pillar Scores
    ──────────────────────────────────────────────
    Perceivable:    22/25
    Operable:       24/25
    Understandable: 25/25
    Robust:         23/25

    Total errors:   2
    Total warnings: 3
  ════════════════════════════════════════════════
```

---

## `axis fix`

Preview the score improvement you would get if every detected issue were fixed — without changing anything.

```
axis fix <TARGET> [OPTIONS]
```

### Arguments

| Argument | Description |
|---|---|
| `<TARGET>` | A URL (`https://example.com`) or a local file path (`./index.html`) |

### Options

| Flag | Default | Description |
|---|---|---|
| `--headless` | off | Render via headless Chrome before running checks |

### Examples

```bash
# Fix preview for a URL
axis fix https://example.com

# Fix preview with full JS rendering
axis fix https://example.com --headless

# Fix preview for a local file
axis fix ./index.html
```

### How scoring works

| Issue type | Score contribution |
|---|---|
| Error | Full `score_impact` added back |
| Warning | Half `score_impact` added back |

### Sample output

```
  ════════════════════════════════════════════════
     AXIS  Fix Preview
  ════════════════════════════════════════════════
    URL: https://example.com

    Score before:  68/100
    Score after:   91/100  (+23)

    Pillar improvement after fixes
    ──────────────────────────────────────────────
    Perceivable:    4/25   →  would improve
    Operable:       24/25  →  would improve
    Understandable: 25/25  →  would improve
    Robust:         15/25  →  would improve

    3 issue(s) fixable
    ──────────────────────────────────────────────

    ✓ 1.1.1 Non-text Content  (+10 score)
      2 image(s) missing alt attribute
      Fix:      Add alt="description" to all img elements
      Location: img[13] src="admin-panel/images/new.gif"
      Impact:   Screen reader users hear nothing when encountering these images.

    ✓ 1.3.5 Identify Input Purpose  (+10 score)
      4 input(s) missing associated label
      Fix:      Add <label for="id"> or aria-label to each input
      Location: input[name="sm11"] (+3 more)
      Impact:   Screen reader users cannot determine what information to enter.

    ✓ 2.4.1 Bypass Blocks  (+3 score)
      No skip navigation link found
      Fix:      Add <a href="#main">Skip to main content</a> as first element
      Location: <body> start
      Impact:   Keyboard users must tab through every navigation item on every page load.
  ════════════════════════════════════════════════
```

---

## `axis ci`

CI/CD mode. Runs the same checks as `axis check` but always exits with code `1` if the score is below the threshold — designed to fail a pipeline.

```
axis ci <TARGET> [OPTIONS]
```

### Arguments

| Argument | Description |
|---|---|
| `<TARGET>` | A URL (`https://example.com`) or a local file path (`./index.html`) |

### Options

| Flag | Default | Description |
|---|---|---|
| `--threshold <N>` | `80` | Minimum acceptable score. Exit `1` if score < N |
| `--json` | off | Output results as JSON (useful for GitHub Actions annotations) |
| `--headless` | off | Render via headless Chrome before running checks |

### Examples

```bash
# Default: fail if score < 80
axis ci https://example.com

# Custom threshold
axis ci https://example.com --threshold 90

# JSON output for GitHub Actions
axis ci https://example.com --threshold 80 --json

# Headless + CI
axis ci https://example.com --headless --threshold 85

# Local file in CI
axis ci ./dist/index.html --threshold 80
```

### Exit codes

| Code | Meaning |
|---|---|
| `0` | Score meets or exceeds threshold — pipeline continues |
| `1` | Score is below threshold — pipeline fails |

### Sample output — text

```
  ════════════════════════════════════════════════
     AXIS  CI Check
  ════════════════════════════════════════════════
    URL:       https://example.com
    Score:     77/100  (threshold: 80)
    Tier:      Partially Compliant

    WCAG 2.2 Pillar Scores
    ──────────────────────────────────────────────
    Perceivable:    23/25
    Operable:       14/25
    Understandable: 15/25
    Robust:         25/25

    Issues
    ──────────────────────────────────────────────
    [Error] 3.1.1 Language of Page — HTML element is missing a lang attribute
            Location: https://example.com → <html>
            Fix: Add lang="en" to the <html> element (+10 score)

    FAIL  Score 77 is below threshold 80
  ════════════════════════════════════════════════
```

### Sample output — JSON (`--json`)

```json
{
  "url": "https://example.com",
  "score": 77,
  "tier": "Partially Compliant",
  "threshold": 80,
  "passed": false,
  "errors": 2,
  "warnings": 2,
  "pillars": {
    "perceivable": 23,
    "operable": 14,
    "understandable": 15,
    "robust": 25
  },
  "issues": [
    {
      "severity": "Error",
      "rule": "3.1.1 Language of Page",
      "pillar": "Understandable",
      "message": "HTML element is missing a lang attribute",
      "location": "https://example.com → <html>",
      "fix": "Add lang=\"en\" to the <html> element",
      "score_impact": 10
    }
  ]
}
```

### GitHub Actions integration

```yaml
name: Accessibility

on: [push, pull_request]

jobs:
  a11y:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Download AXIS
        run: |
          curl -L https://github.com/ABHIRAM-CREATOR06/Axis-CLI/releases/latest/download/axis-linux-x64 \
            -o axis && chmod +x axis

      - name: Check accessibility
        run: ./axis ci https://your-staging-url.com --threshold 80 --json
```

---

## Rendering modes

Both `check`, `fix`, and `ci` support two rendering modes:

| Mode | Flag | How it works | Best for |
|---|---|---|---|
| HTTP | _(default)_ | Direct HTTP GET, no JS execution | Static HTML, speed |
| Headless | `--headless` | Full headless Chrome via `axis-render.exe` | SPAs, JS-rendered content |

### Headless requirements

- `axis-render.exe` must be in the same directory as `axis.exe`
- Chrome is downloaded automatically on first run to `%TEMP%\axis-render-chrome`
- Subsequent runs reuse the cached Chrome install (fast)

### Headless behaviour on slow sites

If a site takes longer than 60 seconds to load, `axis-render` captures whatever partial HTML Chrome managed to load and returns it with a warning. The accessibility checks still run on the partial content.

```
  ⠸ Rendering with headless Chrome... 8s
  ⚠  axis-render warning (partial render): Navigation timeout — partial HTML captured
  ✓ Rendered in 62.1s
```

If no HTML was captured at all, AXIS falls back to HTTP automatically:

```
  ⚠  axis-render error: ... — falling back to HTTP
```

---

## Scoring reference

### Overall score

```
score = 100 − Σ(error.score_impact) − Σ(warning.score_impact / 2)
```

Clamped to `[0, 100]`.

### Tiers

| Score | Tier |
|---|---|
| 95–100 | ✅ Fully Compliant |
| 80–94 | 🟦 Mostly Compliant |
| 60–79 | 🟨 Partially Compliant |
| 0–59 | 🟥 Not Compliant |

### Pillar scores

Each pillar starts at 25. Issue penalties are subtracted from the pillar matching the issue's `wcag_pillar` field. Errors deduct their full `score_impact`; warnings deduct half.

---

## All WCAG checks

| Rule | Name | Pillar | Severity | Score impact |
|---|---|---|---|---|
| 1.1.1 | Non-text Content | Perceivable | Error | −10 |
| 1.3.1 | Info and Relationships | Perceivable | Warning | −3 / −4 |
| 1.3.5 | Identify Input Purpose | Perceivable | Error | −10 |
| 1.7.5 | Autocomplete | Understandable | Warning | −3 |
| 2.4.1 | Bypass Blocks | Operable | Warning | −3 |
| 2.4.2 | Page Titled | Operable | Error | −10 |
| 2.4.6 | Link Purpose | Operable | Warning | −3 |
| 2.5.8 | Target Size | Operable | Warning | −3 |
| 3.1.1 | Language of Page | Understandable | Error | −10 |
| 4.1.2 | Name, Role, Value | Robust | Error | −10 |
