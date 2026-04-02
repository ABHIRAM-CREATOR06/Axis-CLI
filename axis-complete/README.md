# AXIS CLI

WCAG 2.2 accessibility checker for the command line.
Scores pages 0–100, breaks down results by WCAG pillar,
shows the human impact of each issue, and integrates into CI/CD.

## Architecture

```
axis (Rust CLI)
    │
    ├── HTTP mode (default)     — plain reqwest, no dependencies
    └── Headless mode           — calls axis-render (.NET + PuppeteerSharp)
            │
            └── returns JSON → axis checks WCAG rules → outputs report
```

## Install

### Option 1 — Build from source (Rust)

```bash
cd axis-complete
cd axis-cli
cargo build --release
cp target/release/axis /usr/local/bin/axis
```

### Option 2 — Cargo install

```bash
cargo install --path axis-cli
```

### Option 3 — With headless Chrome support

Build the .NET renderer too:

```bash
cd axis-renderer/AxisRenderer
dotnet publish -c Release -r linux-x64 --self-contained
cp bin/Release/net9.0/linux-x64/publish/axis-render /usr/local/bin/axis-render
```

## Usage

```bash
# Check a URL
axis check https://example.com

# Check with minimum score threshold
axis check https://example.com --threshold 80

# Check with headless Chrome (JS rendering)
axis check https://example.com --headless

# Check a local file
axis check ./index.html

# Scan entire local project
axis scan ./my-project

# Save project report as JSON
axis scan ./my-project --output report.json

# Preview score improvement
axis fix https://example.com

# CI mode
axis ci https://example.com --threshold 80

# CI mode with JSON output
axis ci https://example.com --threshold 80 --json
```

## Sample output

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
  Understandable  ██████████████████░░░░░░  18/25
  Robust          █████████████████████░░░  21/25

  Summary
  ──────────────────────────────────────────────
  2  errors      1  warnings      7  passed

  Issues
  ──────────────────────────────────────────────

  ✗ ERROR   1.1.1 Non-text Content [Perceivable]
            3 image(s) missing alt attribute
  Location: https://example.com → img[1] src="hero.jpg"
  Fix:      Add alt="description" to all img elements (+10 score)

  Impact: Screen reader users hear nothing when encountering
          these images — the content is completely invisible to them.

════════════════════════════════════════════════
```

## GitHub Actions

Copy `.github/workflows/accessibility.yml` into your project.
It scans your HTML files on every push and PR, uploads a JSON
report as an artifact, and fails PRs if the average score drops below 80.

## Project structure

```
axis-complete/
├── axis-cli/               Rust CLI
│   └── src/
│       ├── main.rs         Entry point, commands
│       ├── checker.rs      WCAG 2.2 checks
│       ├── renderer.rs     HTTP + headless rendering
│       ├── output.rs       Terminal output, plain text report
│       ├── fix.rs          Fix preview command
│       ├── ci.rs           CI mode
│       ├── scanner.rs      Local project scanner
│       └── types.rs        Data types
│
├── axis-renderer/          .NET headless renderer
│   └── AxisRenderer/
│       ├── Program.cs      PuppeteerSharp renderer
│       └── AxisRenderer.csproj
│
└── .github/
    └── workflows/
        └── accessibility.yml
```

## WCAG checks included

| Rule | Description | Pillar |
|---|---|---|
| 1.1.1 | Images missing alt text | Perceivable |
| 1.3.1 | Missing or multiple h1 | Perceivable |
| 1.3.5 | Inputs without labels | Perceivable |
| 2.4.1 | No skip navigation link | Operable |
| 2.4.2 | Missing page title | Operable |
| 2.4.6 | Vague link text | Operable |
| 2.5.8 | Small click targets | Operable |
| 3.1.1 | Missing lang attribute | Understandable |
| 3.3.7 | Missing autocomplete | Understandable |
| 4.1.2 | Buttons with no name | Robust |

## License

GPL-3.0 — Part of the AXIS accessibility toolchain
https://github.com/ABHIRAM-CREATOR06/Acess1
