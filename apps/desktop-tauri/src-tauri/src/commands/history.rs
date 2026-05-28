use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::canonical_provider_arg;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UsageHistoryPoint {
    pub provider: String,
    pub timestamp: String,
    pub remaining_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SessionLogEntry {
    pub provider: String,
    pub time: String,
    pub model: Option<String>,
    pub tokens: Option<u64>,
    pub directory: Option<String>,
}

#[tauri::command]
pub fn get_usage_history(provider_id: String) -> Result<Vec<UsageHistoryPoint>, String> {
    let provider = canonical_provider_arg(&provider_id)?;
    let path = codexbar_home_dir()?.join("history").join(format!("{provider}.jsonl"));
    read_usage_history(&path, &provider)
}

#[tauri::command]
pub fn get_provider_sessions(provider_id: String) -> Result<Vec<SessionLogEntry>, String> {
    let provider = canonical_provider_arg(&provider_id)?;
    match provider.as_str() {
        "claude" => {
            let path = user_home_dir()?.join(".claude").join("history.jsonl");
            read_claude_sessions(&path)
        }
        _ => Ok(Vec::new()),
    }
}

fn user_home_dir() -> Result<PathBuf, String> {
    if let Some(path) = env::var_os("HOME") {
        return Ok(PathBuf::from(path));
    }
    if let Some(path) = env::var_os("USERPROFILE") {
        return Ok(PathBuf::from(path));
    }
    match (env::var_os("HOMEDRIVE"), env::var_os("HOMEPATH")) {
        (Some(drive), Some(path)) => Ok(PathBuf::from(drive).join(path)),
        _ => Err("Unable to resolve the current user's home directory".to_string()),
    }
}

fn codexbar_home_dir() -> Result<PathBuf, String> {
    Ok(user_home_dir()?.join(".codexbar-zoey"))
}

fn read_usage_history(path: &Path, provider: &str) -> Result<Vec<UsageHistoryPoint>, String> {
    let Some(reader) = open_reader(path)? else {
        return Ok(Vec::new());
    };
    Ok(parse_usage_history_reader(reader, provider))
}

fn read_claude_sessions(path: &Path) -> Result<Vec<SessionLogEntry>, String> {
    let Some(reader) = open_reader(path)? else {
        return Ok(Vec::new());
    };
    Ok(parse_claude_sessions_reader(reader))
}

fn open_reader(path: &Path) -> Result<Option<BufReader<File>>, String> {
    if !path.exists() {
        return Ok(None);
    }
    File::open(path)
        .map(BufReader::new)
        .map(Some)
        .map_err(|err| format!("Failed to read {}: {err}", path.display()))
}

fn parse_usage_history_reader<R: BufRead>(reader: R, provider: &str) -> Vec<UsageHistoryPoint> {
    reader
        .lines()
        .map_while(Result::ok)
        .filter_map(|line| parse_usage_history_line(&line, provider))
        .collect()
}

fn parse_usage_history_line(line: &str, provider: &str) -> Option<UsageHistoryPoint> {
    let value: Value = serde_json::from_str(line).ok()?;
    let timestamp = string_path(&value, &["timestamp", "time", "ts", "created_at", "recorded_at"])?;
    let remaining_pct = number_path(&value, &["remaining_pct", "remainingPercent", "remaining"])
        .or_else(|| {
            let used = number_path(&value, &["used", "usage.used", "usage.used_percent"])?;
            let limit = number_path(&value, &["limit", "usage.limit"])?;
            if limit <= 0.0 {
                return None;
            }
            Some(100.0 - (used / limit * 100.0))
        })?;

    Some(UsageHistoryPoint {
        provider: provider.to_string(),
        timestamp,
        remaining_pct: remaining_pct.clamp(0.0, 100.0),
    })
}

fn parse_claude_sessions_reader<R: BufRead>(reader: R) -> Vec<SessionLogEntry> {
    reader
        .lines()
        .map_while(Result::ok)
        .filter_map(|line| parse_claude_session_line(&line))
        .collect()
}

fn parse_claude_session_line(line: &str) -> Option<SessionLogEntry> {
    let value: Value = serde_json::from_str(line).ok()?;
    let time = string_path(&value, &["timestamp", "created_at", "started_at", "time"])?;
    let model = string_path(&value, &["model", "session.model", "request.model", "metadata.model"]);
    let tokens = u64_path(
        &value,
        &[
            "usage.total_tokens",
            "usage.totalTokens",
            "total_tokens",
            "token_count",
            "tokens",
        ],
    );
    let directory = string_path(
        &value,
        &[
            "cwd",
            "working_dir",
            "directory",
            "project_path",
            "path",
        ],
    );

    Some(SessionLogEntry {
        provider: "claude".to_string(),
        time,
        model,
        tokens,
        directory,
    })
}

fn string_path(value: &Value, paths: &[&str]) -> Option<String> {
    paths.iter().find_map(|path| {
        value_at_path(value, path)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .map(ToString::to_string)
    })
}

fn number_path(value: &Value, paths: &[&str]) -> Option<f64> {
    paths.iter().find_map(|path| {
        let next = value_at_path(value, path)?;
        if let Some(number) = next.as_f64() {
            return Some(number);
        }
        next.as_str()?.trim().parse::<f64>().ok()
    })
}

fn u64_path(value: &Value, paths: &[&str]) -> Option<u64> {
    paths.iter().find_map(|path| {
        let next = value_at_path(value, path)?;
        if let Some(number) = next.as_u64() {
            return Some(number);
        }
        if let Some(number) = next.as_i64() {
            return u64::try_from(number).ok();
        }
        if let Some(number) = next.as_f64() {
            if number.is_sign_negative() {
                return None;
            }
            return Some(number.round() as u64);
        }
        next.as_str()?.trim().parse::<u64>().ok()
    })
}

fn value_at_path<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = value;
    for key in path.split('.') {
        current = current.get(key)?;
    }
    Some(current)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn parses_usage_history_from_jsonl() {
        let reader = Cursor::new(
            "{\"timestamp\":\"2026-05-28T10:00:00Z\",\"remaining_pct\":82}\n"
                .to_string()
                + "{\"time\":\"2026-05-28T11:00:00Z\",\"used\":18,\"limit\":100}\n",
        );

        let points = parse_usage_history_reader(reader, "claude");
        assert_eq!(points.len(), 2);
        assert_eq!(points[0].provider, "claude");
        assert_eq!(points[0].remaining_pct, 82.0);
        assert_eq!(points[1].remaining_pct, 82.0);
    }

    #[test]
    fn parses_claude_session_rows() {
        let reader = Cursor::new(
            "{\"timestamp\":\"2026-05-28T09:15:00Z\",\"model\":\"claude-3.7-sonnet\",\"usage\":{\"total_tokens\":1200},\"cwd\":\"E:/repo\"}\n",
        );

        let sessions = parse_claude_sessions_reader(reader);
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].provider, "claude");
        assert_eq!(sessions[0].model.as_deref(), Some("claude-3.7-sonnet"));
        assert_eq!(sessions[0].tokens, Some(1200));
        assert_eq!(sessions[0].directory.as_deref(), Some("E:/repo"));
    }
}
