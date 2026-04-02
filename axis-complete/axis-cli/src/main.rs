use clap::{Parser, Subcommand};
use colored::*;

mod checker;
mod ci;
mod fix;
mod output;
mod renderer;
mod scanner;
mod types;

use checker::{calculate_pillars, calculate_score, get_tier, run_checks};
use ci::run_ci;
use fix::run_fix_preview;
use output::{print_json, print_text};
use renderer::{render, render_file, RenderMode};
use scanner::scan_local;
use types::{Report, RenderedPage, Summary};

#[derive(Parser)]
#[command(
    name = "axis",
    about = "WCAG 2.2 accessibility checker — fast, precise, human",
    version = "0.2.0",
    author = "Abhiram",
    long_about = "
AXIS checks websites and local HTML files against WCAG 2.2 standards.
It scores pages from 0–100, breaks down results by WCAG pillar,
shows the human impact of each issue, and integrates into CI/CD pipelines.

ARCHITECTURE:
  axis (Rust CLI) → axis-render (.NET, optional) → AXIS-CORE checks

Place axis-render(.exe) in the same directory as axis to enable
full JavaScript rendering via headless Chrome.
"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check a URL or local HTML file for accessibility issues
    Check {
        /// URL (https://example.com) or local file path (./index.html)
        target: String,

        /// Fail if score is below this threshold
        #[arg(long, default_value = "0")]
        threshold: u32,

        /// Output format: text (default) or json
        #[arg(long, default_value = "text")]
        format: String,

        /// Save report to file
        #[arg(long)]
        output: Option<String>,

        /// Use headless Chrome via axis-render for JS rendering
        #[arg(long)]
        headless: bool,
    },

    /// Scan all HTML files in a local project directory
    Scan {
        /// Path to project directory
        path: String,

        /// Output format: text (default) or json
        #[arg(long, default_value = "text")]
        format: String,

        /// Save project report to file
        #[arg(long)]
        output: Option<String>,
    },

    /// Preview score improvement if all issues were fixed
    Fix {
        /// URL or local file path
        target: String,

        /// Use headless Chrome via axis-render
        #[arg(long)]
        headless: bool,
    },

    /// Run in CI mode — exits with code 1 if score is below threshold
    Ci {
        /// URL or local file path
        target: String,

        /// Minimum acceptable score
        #[arg(long, default_value = "80")]
        threshold: u32,

        /// Output as JSON (useful for GitHub Actions annotations)
        #[arg(long)]
        json: bool,

        /// Use headless Chrome via axis-render
        #[arg(long)]
        headless: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check {
            target,
            threshold,
            format,
            output,
            headless,
        } => {
            let page = fetch_page(&target, headless);
            let report = build_report(&target, &page);
            match format.as_str() {
                "json" => print_json(&report, output),
                _      => print_text(&report, output),
            }
            if report.score < threshold {
                eprintln!(
                    "  {}  Score {} is below threshold {}",
                    "FAIL".red().bold(),
                    report.score,
                    threshold
                );
                std::process::exit(1);
            }
        }

        Commands::Scan { path, format, output } => {
            scan_local(&path, &format, output);
        }

        Commands::Fix { target, headless } => {
            let page = fetch_page(&target, headless);
            let report = build_report(&target, &page);
            run_fix_preview(&report);
        }

        Commands::Ci { target, threshold, json, headless } => {
            let page = fetch_page(&target, headless);
            let report = build_report(&target, &page);
            run_ci(&report, threshold, json);
        }
    }
}

fn fetch_page(target: &str, headless: bool) -> RenderedPage {
    if target.starts_with("http://") || target.starts_with("https://") {
        if headless {
            println!(
                "\n  {} {}",
                "Rendering with headless Chrome:".dimmed(),
                target.cyan()
            );
            match render(target, RenderMode::Headless) {
                Ok(page) => page,
                Err(e) => {
                    eprintln!(
                        "  {}  {} — falling back to HTTP\n",
                        "⚠".yellow(),
                        e
                    );
                    render(target, RenderMode::Http).unwrap_or_else(|e| {
                        eprintln!("  HTTP fallback also failed: {}", e);
                        std::process::exit(1);
                    })
                }
            }
        } else {
            println!(
                "\n  {} {}",
                "Scanning accessibility for".dimmed(),
                target.cyan()
            );
            render(target, RenderMode::Http).unwrap_or_else(|e| {
                eprintln!("  Failed to fetch {}: {}", target, e);
                std::process::exit(1);
            })
        }
    } else {
        println!(
            "\n  {} {}",
            "Checking local file:".dimmed(),
            target.cyan()
        );
        render_file(target).unwrap_or_else(|e| {
            eprintln!("  Failed to read {}: {}", target, e);
            std::process::exit(1);
        })
    }
}

pub fn build_report(target: &str, page: &RenderedPage) -> Report {
    let issues = run_checks(&page.html, target);
    let score = calculate_score(&issues);
    let tier = get_tier(score).to_string();
    let errors = issues.iter().filter(|i| i.severity == "Error").count() as u32;
    let warnings = issues.iter().filter(|i| i.severity == "Warning").count() as u32;
    let total_checks = 10u32;
    let passed = total_checks.saturating_sub(errors + warnings);
    let pillars = calculate_pillars(&issues);

    Report {
        url: target.to_string(),
        score,
        tier,
        issues,
        summary: Summary { errors, warnings, passed },
        pillars,
        load_time_ms: page.load_time_ms,
        total_bytes: page.total_bytes,
        request_count: page.request_count,
    }
}
