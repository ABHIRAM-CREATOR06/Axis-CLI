# AXIS — Web Accessibility Checker

> **Score any website against WCAG 2.2 in seconds.**  
> Fast Rust CLI · Headless Chrome rendering · CI/CD ready · Human-first output

```
  ✓ Rendered in 0.3s

  ════════════════════════════════════════════════
     AXIS  Web Accessibility Report
  ════════════════════════════════════════════════
    URL:    https://news.ycombinator.com
    Score:  77/100   Tier: Partially Compliant

    Perceivable      ██████████████████████░░  23/25
    Operable         █████████████░░░░░░░░░░░  14/25
    Understandable   ██████████████░░░░░░░░░░  15/25
    Robust           ████████████████████████  25/25
  ════════════════════════════════════════════════
```

---

## What is AXIS?

AXIS is a command-line accessibility checker built in Rust. It fetches a page — either over HTTP or via a full headless Chrome render — runs it through 10 WCAG 2.2 checks, and gives you a score from 0–100 broken down by pillar, with plain-English explanations of every issue and exactly how to fix it.

It's designed to be fast enough to run in a pre-commit hook and precise enough to trust in a CI pipeline.

---

## Architecture

```
axis (Rust CLI)
  │
  ├── HTTP mode        — direct fetch, instant, no JS execution
  │
  └── Headless mode    — spawns axis-render (.NET / PuppeteerSharp)
                              │
                              └── Headless Chrome
                                    │
                                    └── Full JS-rendered HTML → AXIS checks
```

`axis-render` is an optional companion binary. Drop it next to `axis.exe` to unlock `--headless`.

---

## Installation

### Pre-built binaries

Download the latest release from the [Releases](https://github.com/ABHIRAM-CREATOR06/Axis-CLI/releases) page.

| File | Platform |
|---|---|
| `axis.exe` | Windows x64 |
| `axis-render.exe` | Windows x64 (optional, for `--headless`) |

Place both in the same directory and add it to your `PATH`.

### Build from source

**Requirements:** Rust 1.75+, .NET 9 SDK (for axis-render only)

```bash
# Clone
git clone https://github.com/ABHIRAM-CREATOR06/Axis-CLI.git
cd Axis-CLI/axis-complete/axis-cli

# Build axis
cargo build --release

# Build axis-render (optional)
cd ../axis-renderer/AxisRenderer
dotnet publish -c Release -r win-x64 --self-contained true -p:PublishSingleFile=true
```

---

## Usage

### `check` — Scan a URL or local file

```bash
# HTTP mode (fast, no JS)
axis check https://example.com

# Headless Chrome (full JS rendering)
axis check https://example.com --headless

# Local HTML file
axis check ./index.html

# JSON output
axis check https://example.com --format json

# Save report to file
axis check https://example.com --output report.txt

# Fail if score is below threshold
axis check https://example.com --threshold 80
```

### `scan` — Scan an entire local project

```bash
axis scan ./my-website/

# Save project report
axis scan ./my-website/ --output project-report.json
```

### `fix` — Preview score improvement if issues were fixed

```bash
axis fix https://example.com
axis fix ./index.html --headless
```

```
  ════════════════════════════════════════════════
     AXIS  Fix Preview
  ════════════════════════════════════════════════
    Score before:  68/100
    Score after:   91/100  (+23)

    ✓ 1.1.1 Non-text Content        (+10 score)
      Fix: Add alt="description" to all img elements
      Impact: Screen reader users hear nothing when encountering these images.

    ✓ 1.3.5 Identify Input Purpose  (+10 score)
      Fix: Add <label for="id"> or aria-label to each input
```

### `ci` — CI/CD integration

```bash
# Exits with code 1 if score < threshold
axis ci https://example.com --threshold 80

# JSON output for GitHub Actions annotations
axis ci https://example.com --threshold 80 --json
```

---

## WCAG 2.2 Checks

AXIS runs 10 checks across all four WCAG pillars, each scored out of 25:

| Rule | Pillar | Severity | Impact |
|---|---|---|---|
| 1.1.1 Non-text Content | Perceivable | Error | Images missing `alt` attributes |
| 1.3.1 Info and Relationships | Perceivable | Warning | Missing or multiple `h1` headings |
| 1.3.5 Identify Input Purpose | Perceivable | Error | Inputs without labels |
| 2.4.1 Bypass Blocks | Operable | Warning | No skip navigation link |
| 2.4.2 Page Titled | Operable | Error | Missing or empty `<title>` |
| 2.4.6 Link Purpose | Operable | Warning | Vague link text ("click here") |
| 2.5.8 Target Size | Operable | Warning | Interactive elements too small |
| 3.1.1 Language of Page | Understandable | Error | Missing `lang` attribute on `<html>` |
| 1.7.5 Autocomplete | Understandable | Warning | Personal inputs missing `autocomplete` |
| 4.1.2 Name, Role, Value | Robust | Error | Buttons with no accessible name |

### Scoring

| Score | Tier |
|---|---|
| 95–100 | ✅ Fully Compliant |
| 80–94 | 🟦 Mostly Compliant |
| 60–79 | 🟨 Partially Compliant |
| 0–59 | 🟥 Not Compliant |

- **Errors** deduct their full `score_impact` from the total
- **Warnings** deduct half their `score_impact`
- Each pillar is scored out of 25; penalties are applied per-pillar based on the issue's WCAG category

---

## CI/CD Integration

### GitHub Actions

```yaml
name: Accessibility Check

on: [push, pull_request]

jobs:
  accessibility:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Download AXIS
        run: |
          curl -L https://github.com/ABHIRAM-CREATOR06/Axis-CLI/releases/latest/download/axis-linux-x64 \
            -o axis && chmod +x axis

      - name: Run accessibility check
        run: ./axis ci https://your-site.com --threshold 80 --json
```

### Pre-commit hook

```bash
#!/bin/sh
# .git/hooks/pre-commit
axis ci ./index.html --threshold 80
if [ $? -ne 0 ]; then
  echo "Accessibility check failed. Fix issues before committing."
  exit 1
fi
```

---

## Output Formats

### Text (default)

Coloured, human-readable terminal output with progress bars, issue details, fix suggestions, and persona impact hints.

### JSON (`--format json`)

```json
{
  "url": "https://example.com",
  "score": 77,
  "tier": "Partially Compliant",
  "pillars": {
    "perceivable": 23,
    "operable": 14,
    "understandable": 15,
    "robust": 25
  },
  "summary": { "errors": 2, "warnings": 2, "passed": 6 },
  "issues": [
    {
      "severity": "Error",
      "rule": "3.1.1 Language of Page",
      "wcag_pillar": "Understandable",
      "message": "HTML element is missing a lang attribute",
      "location": "https://example.com → <html>",
      "fix": "Add lang=\"en\" to the <html> element",
      "score_impact": 10,
      "persona_hint": "Screen readers cannot switch to the correct pronunciation engine..."
    }
  ]
}
```

---

## Headless Mode

When you pass `--headless`, AXIS spawns `axis-render.exe` which uses PuppeteerSharp to drive a real headless Chrome instance. This means:

- JavaScript is fully executed before the HTML is captured
- Single-page applications (React, Vue, Angular) are checked against their rendered output, not their source
- Network requests and page load time are recorded

```bash
axis check https://my-react-app.com --headless
```

```
  ⠸ Rendering with headless Chrome... 4s
  ✓ Rendered in 6.2s

    Load time: 6241ms
    Page size: 142kb
    Requests:  47
```

`axis-render` downloads Chrome automatically on first run. Subsequent runs reuse the cached install.

---

## Project Structure

```
Axis-CLI/
├── axis-complete/
│   ├── axis-cli/               # Rust CLI (axis.exe)
│   │   └── src/
│   │       ├── main.rs         # CLI entry point & argument parsing
│   │       ├── checker.rs      # WCAG 2.2 check implementations
│   │       ├── renderer.rs     # HTTP + headless rendering
│   │       ├── scanner.rs      # Local project directory scanner
│   │       ├── output.rs       # Text & JSON report formatting
│   │       ├── fix.rs          # Fix preview mode
│   │       ├── ci.rs           # CI/CD mode
│   │       └── types.rs        # Shared data types
│   │
│   └── axis-renderer/          # .NET companion (axis-render.exe)
│       └── AxisRenderer/
│           └── Program.cs      # PuppeteerSharp headless Chrome runner
```

---

## Contributing

Issues and PRs are welcome. When adding a new WCAG check:

1. Implement it in `checker.rs` following the existing pattern
2. Add it to `run_checks()`
3. Update `total_checks` in `main.rs` and `scanner.rs`
4. Add unit tests in the `#[cfg(test)]` block at the bottom of `checker.rs`

---

## License

MIT — see [LICENSE](LICENSE) for details.

---

*Built by [Abhiram](https://github.com/ABHIRAM-CREATOR06)*
