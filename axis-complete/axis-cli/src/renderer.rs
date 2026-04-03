use crate::types::RenderedPage;
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub enum RenderMode {
    Headless,
    Http,
}

pub fn render(url: &str, mode: RenderMode) -> Result<RenderedPage, String> {
    match mode {
        RenderMode::Headless => render_headless(url),
        RenderMode::Http => render_http(url),
    }
}

fn render_headless(url: &str) -> Result<RenderedPage, String> {
    let renderer = std::env::current_exe()
        .map(|p| {
            p.parent().unwrap().join(if cfg!(windows) {
                "axis-render.exe"
            } else {
                "axis-render"
            })
        })
        .unwrap_or_else(|_| "axis-render".into());

    // Pre-flight: give a clear message if the binary is simply missing.
    if !renderer.exists() {
        return Err(format!(
            "axis-render not found at {}. \
             Place axis-render(.exe) next to axis(.exe), \
             or run without --headless for HTTP mode.",
            renderer.display()
        ));
    }

    let mut child = Command::new(&renderer)
        .arg("--url")
        .arg(url)
        .arg("--json")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to launch {}: {}", renderer.display(), e))?;

    // ── Spinner + hard timeout ────────────────────────────────────────────────
    let spinner = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let start = Instant::now();
    let timeout_secs = 75u64; // 60s for the page + 15s headroom for Chrome launch
    let mut tick = 0usize;

    loop {
        match child.try_wait().map_err(|e| e.to_string())? {
            Some(_) => break,
            None => {
                let elapsed = start.elapsed().as_secs();
                if elapsed >= timeout_secs {
                    child.kill().ok();
                    eprint!("\r{}\r", " ".repeat(70));
                    let _ = std::io::stderr().flush();
                    return Err(format!(
                        "axis-render timed out after {}s — \
                         try again or run without --headless",
                        timeout_secs
                    ));
                }
                eprint!(
                    "\r  {} Rendering with headless Chrome... {}s  ",
                    spinner[tick % spinner.len()],
                    elapsed
                );
                let _ = std::io::stderr().flush();
                tick += 1;
                std::thread::sleep(Duration::from_millis(100));
            }
        }
    }

    eprint!("\r{}\r", " ".repeat(70));
    let _ = std::io::stderr().flush();

    // ── Read output (process already exited via try_wait) ─────────────────────
    let mut stdout_buf = Vec::new();
    let mut stderr_buf = Vec::new();

    if let Some(mut h) = child.stdout.take() {
        h.read_to_end(&mut stdout_buf).ok();
    }
    if let Some(mut h) = child.stderr.take() {
        h.read_to_end(&mut stderr_buf).ok();
    }

    // ── If stdout is empty it's a genuine crash (missing DLL, etc.) ───────────
    if stdout_buf.is_empty() {
        let stderr_text = String::from_utf8_lossy(&stderr_buf);
        let cause = stderr_text
            .lines()
            .filter(|l| !l.trim().is_empty())
            .last()
            .unwrap_or("no output produced — check axis-render.exe is the correct build")
            .to_string();
        return Err(format!(
            "axis-render crashed: {}.\n  \
             Run `axis-render.exe --url {} --json` directly to see the full error.",
            cause, url
        ));
    }

    // ── Parse JSON ────────────────────────────────────────────────────────────
    let page: RenderedPage = serde_json::from_slice(&stdout_buf)
        .map_err(|e| format!("Failed to parse renderer output: {}", e))?;

    // ── Check the error field in the JSON ─────────────────────────────────────
    // Program.cs always exits 0 and puts errors in the `error` JSON field so
    // we can distinguish a real crash (empty stdout) from a handled error.
    if let Some(ref err) = page.error {
        if page.html.is_empty() {
            // Navigation failed and no partial HTML was captured — give up and
            // let the caller fall back to HTTP.
            return Err(format!("axis-render error: {}", err));
        }
        // Partial HTML was captured despite the error — log a warning and
        // continue; the accessibility checks will still be useful.
        eprintln!("  ⚠  axis-render warning (partial render): {}", err);
    }

    eprintln!("  ✓ Rendered in {:.1}s", start.elapsed().as_secs_f32());
    Ok(page)
}

fn render_http(url: &str) -> Result<RenderedPage, String> {
    let response = reqwest::blocking::Client::new()
        .get(url)
        .header("User-Agent", "axis-cli/0.2.0 accessibility-checker")
        .send()
        .map_err(|e| format!("HTTP fetch failed: {}", e))?;

    let total_bytes = response.content_length().unwrap_or(0);
    let html = response
        .text()
        .map_err(|e| format!("Failed to read response: {}", e))?;

    Ok(RenderedPage {
        html,
        title: String::new(),
        load_time_ms: 0,
        total_bytes,
        request_count: 1,
        requests: vec![],
        screenshot_base64: None,
        error: None,
    })
}

pub fn render_file(path: &str) -> Result<RenderedPage, String> {
    let html = std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

    Ok(RenderedPage {
        html,
        title: String::new(),
        load_time_ms: 0,
        total_bytes: 0,
        request_count: 0,
        requests: vec![],
        screenshot_base64: None,
        error: None,
    })
}
