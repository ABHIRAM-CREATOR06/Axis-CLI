use crate::types::RenderedPage;
use std::process::Command;

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
            p.parent()
                .unwrap()
                .join(if cfg!(windows) { "axis-render.exe" } else { "axis-render" })
        })
        .unwrap_or_else(|_| "axis-render".into());

    let output = Command::new(&renderer)
        .arg("--url")
        .arg(url)
        .arg("--json")
        .output()
        .map_err(|e| format!(
            "axis-render not found: {}. \
             Place axis-render(.exe) in the same directory as axis, \
             or run without --headless for HTTP mode.",
            e
        ))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("axis-render failed: {}", err));
    }

    serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse renderer output: {}", e))
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
    let html = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

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
