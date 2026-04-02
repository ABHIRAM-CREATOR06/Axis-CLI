# AXIS CLI Commands

## check

Check a URL or local HTML file for accessibility issues.

```bash
axis check <target> [options]
```

### Arguments

| Argument | Description |
|----------|-------------|
| `target` | URL (https://example.com) or local file path (./index.html) |

### Options

| Option | Description | Default |
|--------|-------------|---------|
| `--threshold <num>` | Fail if score is below this threshold | 0 |
| `--format <format>` | Output format: text or json | text |
| `--output <file>` | Save report to file | - |
| `--headless` | Use headless Chrome for JS rendering | false |

### Examples

```bash
# Check a URL
axis check https://example.com

# Check with minimum score threshold (fails if below 80)
axis check https://example.com --threshold 80

# Check with headless Chrome (renders JavaScript)
axis check https://example.com --headless

# Check a local file
axis check ./index.html

# Output as JSON
axis check https://example.com --format json

# Save report to file
axis check https://example.com --output report.json
```

---

## scan

Scan all HTML files in a local project directory.

```bash
axis scan <path> [options]
```

### Arguments

| Argument | Description |
|----------|-------------|
| `path` | Path to project directory |

### Options

| Option | Description | Default |
|--------|-------------|---------|
| `--format <format>` | Output format: text or json | text |
| `--output <file>` | Save project report to file | - |

### Examples

```bash
# Scan entire project
axis scan ./my-project

# Scan and output as JSON
axis scan ./my-project --format json

# Save project report
axis scan ./my-project --output report.json
```

---

## fix

Preview score improvement if all accessibility issues were fixed.

```bash
axis fix <target> [options]
```

### Arguments

| Argument | Description |
|----------|-------------|
| `target` | URL or local file path |

### Options

| Option | Description | Default |
|--------|-------------|---------|
| `--headless` | Use headless Chrome for JS rendering | false |

### Examples

```bash
# Preview fixes for a URL
axis fix https://example.com

# Preview fixes with headless Chrome
axis fix https://example.com --headless
```

---

## ci

Run in CI mode. Exits with code 1 if score is below threshold.

```bash
axis ci <target> [options]
```

### Arguments

| Argument | Description |
|----------|-------------|
| `target` | URL or local file path |

### Options

| Option | Description | Default |
|--------|-------------|---------|
| `--threshold <num>` | Minimum acceptable score | 80 |
| `--json` | Output as JSON | false |
| `--headless` | Use headless Chrome for JS rendering | false |

### Examples

```bash
# CI mode (fails if below 80)
axis ci https://example.com

# CI mode with custom threshold
axis ci https://example.com --threshold 90

# CI mode with JSON output (for GitHub Actions)
axis ci https://example.com --json

# CI mode with headless Chrome
axis ci https://example.com --headless
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success — score meets threshold |
| 1 | Failure — score below threshold |

---

## GitHub Actions Example

```yaml
name: Accessibility

on: [push, pull_request]

jobs:
  accessibility:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run AXIS
        uses: docker://abhiram-creator06/axis-cli:latest
        with:
          args: "ci https://example.com --threshold 80"
      - name: Upload report
        if: failure()
        uses: actions/upload-artifact@v4
        with:
          name: accessibility-report
          path: report.json
```

---

## Output Formats

### Text (default)

```
════════════════════════════════════════════════
   AXIS  Web Accessibility Report
════════════════════════════════════════════════
  URL:   https://example.com
  Score: 74/100
  Tier:  Partially Compliant

  WCAG 2.2 Pillar Scores
  ──────────────────────────────────────────────
  Perceivable     ████████████░░░░░░░░░░░░  15/25
  Operable        ████████████████████░░░░  20/25
  Understandable  ██████████████████░░░░░░░  18/25
  Robust          █████████████████████░░░░  21/25

  Summary
  ──────────────────────────────────────────────
  2  errors      1  warnings      7  passed
════════════════════════════════════════════════
```

### JSON

```json
{
  "url": "https://example.com",
  "score": 74,
  "tier": "Partially Compliant",
  "issues": [
    {
      "id": "1.1.1",
      "description": "Non-text Content",
      "severity": "Error",
      "pillar": "Perceivable",
      "count": 3,
      "impact": "Screen reader users hear nothing...",
      "fix": "Add alt=\"description\" to all img elements"
    }
  ],
  "summary": {
    "errors": 2,
    "warnings": 1,
    "passed": 7
  },
  "pillars": {
    "perceivable": 15,
    "operable": 20,
    "understandable": 18,
    "robust": 21
  }
}
```