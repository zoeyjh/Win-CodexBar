use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use codexbar::core::ProviderError;
use serde::{Deserialize, Serialize};

use crate::bar::data_root_dir;
use crate::commands::ProviderUsageSnapshot;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UsageUpdateSnapshot {
    pub provider: String,
    pub plan: Option<String>,
    pub unit: String,
    pub used: Option<f64>,
    pub limit: Option<f64>,
    pub remaining_pct: Option<f64>,
    pub reset_at: Option<String>,
    pub status: String,
    pub last_success_at: String,
    pub confidence: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsagePollStatus {
    Ok,
    RateLimited,
    Network,
    AuthExpired,
    Unknown,
}

impl UsagePollStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::RateLimited => "rate_limited",
            Self::Network => "network",
            Self::AuthExpired => "auth_expired",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Serialize)]
struct UsageHistoryEntry<'a> {
    ts: &'a str,
    provider: &'a str,
    remaining_pct: Option<f64>,
    used: Option<f64>,
    limit: Option<f64>,
}

pub fn classify_provider_error(error: &ProviderError) -> UsagePollStatus {
    match error {
        ProviderError::AuthRequired => UsagePollStatus::AuthExpired,
        ProviderError::Network(_) | ProviderError::Timeout => UsagePollStatus::Network,
        ProviderError::OAuth(message) | ProviderError::Other(message) | ProviderError::Parse(message) => {
            classify_message(message)
        }
        ProviderError::NotInstalled(_) | ProviderError::NoCookies => UsagePollStatus::AuthExpired,
        ProviderError::UnsupportedSource(_) => UsagePollStatus::Unknown,
    }
}

fn classify_message(message: &str) -> UsagePollStatus {
    let lower = message.trim().to_ascii_lowercase();

    if lower.contains("429") || lower.contains("rate limit") || lower.contains("too many requests") {
        return UsagePollStatus::RateLimited;
    }

    if lower.contains("401")
        || lower.contains("403")
        || lower.contains("auth required")
        || lower.contains("authentication required")
        || lower.contains("token invalid")
        || lower.contains("token expired")
        || lower.contains("credentials not found")
        || lower.contains("sign-in")
        || lower.contains("sign in")
        || lower.contains("forbidden")
        || lower.contains("unauthorized")
    {
        return UsagePollStatus::AuthExpired;
    }

    if lower.contains("timeout")
        || lower.contains("timed out")
        || lower.contains("request failed")
        || lower.contains("network")
        || lower.contains("dns")
        || lower.contains("connection")
        || lower.contains("502")
        || lower.contains("503")
        || lower.contains("504")
        || lower.contains("500")
    {
        return UsagePollStatus::Network;
    }

    UsagePollStatus::Unknown
}

pub fn usage_snapshot_from_provider(snapshot: &ProviderUsageSnapshot) -> UsageUpdateSnapshot {
    UsageUpdateSnapshot {
        provider: snapshot.provider_id.clone(),
        plan: snapshot.plan_name.clone(),
        unit: "percent".to_string(),
        used: Some(snapshot.primary.used_percent),
        limit: Some(100.0),
        remaining_pct: Some(snapshot.primary.remaining_percent),
        reset_at: snapshot.primary.resets_at.clone(),
        status: UsagePollStatus::Ok.as_str().to_string(),
        last_success_at: snapshot.updated_at.clone(),
        confidence: "high".to_string(),
    }
}

pub fn usage_snapshot_from_status(
    provider: &str,
    status: UsagePollStatus,
    previous: Option<&UsageUpdateSnapshot>,
    observed_at: &str,
) -> UsageUpdateSnapshot {
    let carry = previous.cloned();
    let preserve_metrics = matches!(status, UsagePollStatus::RateLimited | UsagePollStatus::Network);
    let confidence = if previous.is_some() { "cached" } else { "inferred" };

    UsageUpdateSnapshot {
        provider: provider.to_string(),
        plan: carry.as_ref().and_then(|snapshot| snapshot.plan.clone()),
        unit: carry
            .as_ref()
            .map(|snapshot| snapshot.unit.clone())
            .unwrap_or_else(|| "percent".to_string()),
        used: if preserve_metrics {
            carry.as_ref().and_then(|snapshot| snapshot.used)
        } else {
            None
        },
        limit: if preserve_metrics {
            carry.as_ref().and_then(|snapshot| snapshot.limit)
        } else {
            None
        },
        remaining_pct: if preserve_metrics {
            carry.as_ref().and_then(|snapshot| snapshot.remaining_pct)
        } else {
            None
        },
        reset_at: if preserve_metrics {
            carry.as_ref().and_then(|snapshot| snapshot.reset_at.clone())
        } else {
            None
        },
        status: status.as_str().to_string(),
        last_success_at: carry
            .as_ref()
            .map(|snapshot| snapshot.last_success_at.clone())
            .unwrap_or_else(|| observed_at.to_string()),
        confidence: confidence.to_string(),
    }
}

pub fn upsert_usage_cache(cache: &mut Vec<UsageUpdateSnapshot>, snapshot: UsageUpdateSnapshot) {
    if let Some(existing) = cache
        .iter_mut()
        .find(|existing| existing.provider == snapshot.provider)
    {
        *existing = snapshot;
    } else {
        cache.push(snapshot);
    }
}

pub fn usage_cache_path() -> PathBuf {
    data_root_dir().join("usage-cache.json")
}

pub fn load_usage_cache() -> Result<Vec<UsageUpdateSnapshot>, String> {
    let path = usage_cache_path();
    if !path.exists() {
        return Ok(Vec::new());
    }

    let text = fs::read_to_string(&path).map_err(|err| format!("Failed to read {}: {err}", path.display()))?;
    serde_json::from_str(&text).map_err(|err| format!("Failed to parse {}: {err}", path.display()))
}

pub fn save_usage_cache(snapshots: &[UsageUpdateSnapshot]) -> Result<(), String> {
    let path = usage_cache_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }

    let text = serde_json::to_string_pretty(snapshots).map_err(|err| err.to_string())?;
    fs::write(&path, text).map_err(|err| format!("Failed to write {}: {err}", path.display()))
}

pub fn should_append_usage_history(snapshot: &UsageUpdateSnapshot) -> bool {
    snapshot.remaining_pct.is_some() || (snapshot.used.is_some() && snapshot.limit.is_some())
}

pub fn append_usage_history(snapshot: &UsageUpdateSnapshot, polled_at: &str) -> Result<(), String> {
    let path = data_root_dir()
        .join("history")
        .join(format!("{}.jsonl", snapshot.provider));
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|err| format!("Failed to open {}: {err}", path.display()))?;
    let entry = UsageHistoryEntry {
        ts: polled_at,
        provider: &snapshot.provider,
        remaining_pct: snapshot.remaining_pct,
        used: snapshot.used,
        limit: snapshot.limit,
    };
    let line = serde_json::to_string(&entry).map_err(|err| err.to_string())?;
    writeln!(file, "{line}").map_err(|err| err.to_string())
}
