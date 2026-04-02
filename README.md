# AXIS CLI

WCAG 2.2 accessibility checker for the command line. Scores pages 0–100, breaks down results by WCAG pillar, shows the human impact of each issue, and integrates into CI/CD.

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
cargo install --path axis-complete/axis-cli
```

### Option 3 — With headless Chrome support

Build the .NET renderer too:

```bash
cd axis-complete/axis-renderer/AxisRenderer
dotnet publish -c Release -r linux-x64 --self-contained
cp bin/Release/net9.0/linux-x64/publish/axis-render /usr/local/bin/axis-render
```

## Quick Start

```bash
# Check a URL
axis check https://example.com

# Check with minimum score threshold
axis check https://example.com --threshold 80

# Check with headless Chrome (JS rendering)
axis check https://example.com --headless

# Check a local file
axis check ./index.html
```

## Commands

| Command | Description |
|---------|-------------|
| `check` | Check a URL or local HTML file |
| `scan` | Scan all HTML files in a directory |
| `fix` | Preview score improvement |
| `ci` | CI mode with exit codes |

See [commands.md](commands.md) for detailed usage.

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
  Understandable  ██████████████████░░░░░░░  18/25
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

## Project structure

```
Axis-CLI/
├── axis-complete/           Main project
│   ├── axis-cli/             Rust CLI
│   │   └── src/
│   │       ├── main.rs       Entry point, commands
│   │       ├── checker.rs     WCAG 2.2 checks
│   │       ├── renderer.rs   HTTP + headless rendering
│   │       ├── output.rs     Terminal output
│   │       ├── fix.rs       Fix preview
│   │       ├── ci.rs        CI mode
│   │       ├── scanner.rs   Local scanner
│   │       └── types.rs     Data types
│   │
│   ├── axis-renderer/       .NET headless renderer
│   │   └── AxisRenderer/
│   │       └── Program.cs   PuppeteerSharp
│   │
│   └── .github/
│       └── workflows/       GitHub Actions
│
├── README.md
└── commands.md
```

## License

GPL-3.0 — Part of the AXIS accessibility toolchain