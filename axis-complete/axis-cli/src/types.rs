use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub severity: String,
    pub rule: String,
    pub wcag_pillar: String,
    pub message: String,
    pub location: String,
    pub fix: String,
    pub score_impact: u32,
    pub persona_hint: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PillarScores {
    pub perceivable: u32,
    pub operable: u32,
    pub understandable: u32,
    pub robust: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Summary {
    pub errors: u32,
    pub warnings: u32,
    pub passed: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Report {
    pub url: String,
    pub score: u32,
    pub tier: String,
    pub issues: Vec<Issue>,
    pub summary: Summary,
    pub pillars: PillarScores,
    pub load_time_ms: u64,
    pub total_bytes: u64,
    pub request_count: u32,
}

#[derive(Debug, Deserialize)]
pub struct RequestEntry {
    pub url: String,
    pub size_bytes: u64,
    pub resource_type: String,
}

#[derive(Debug, Deserialize)]
pub struct RenderedPage {
    pub html: String,
    pub title: String,
    pub load_time_ms: u64,
    pub total_bytes: u64,
    pub request_count: u32,
    pub requests: Vec<RequestEntry>,
    pub screenshot_base64: Option<String>,
    pub error: Option<String>,
}
