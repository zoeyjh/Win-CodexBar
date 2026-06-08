use std::collections::HashSet;

use crate::bar::state::BarState;
use crate::commands::ProviderCatalogEntry;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TrayMenuEntry {
    pub(crate) id: Option<String>,
    pub(crate) label: String,
    pub(crate) children: Vec<Self>,
    pub(crate) is_separator: bool,
    pub(crate) disabled: bool,
    pub(crate) checked: Option<bool>,
}

impl TrayMenuEntry {
    fn item(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: Some(id.into()),
            label: label.into(),
            children: Vec::new(),
            is_separator: false,
            disabled: false,
            checked: None,
        }
    }

    fn separator() -> Self {
        Self {
            id: None,
            label: String::new(),
            children: Vec::new(),
            is_separator: true,
            disabled: false,
            checked: None,
        }
    }

    fn path_segment(&self) -> Option<String> {
        if self.is_separator {
            return None;
        }

        Some(
            self.id
                .clone()
                .unwrap_or_else(|| self.label.to_ascii_lowercase().replace(' ', "_")),
        )
    }
}

pub(crate) fn build_tray_menu(
    providers: &[ProviderCatalogEntry],
    status_labels: &[(String, String)],
    enabled_providers: &HashSet<String>,
    bar_state: BarState,
) -> Vec<TrayMenuEntry> {
    build_tray_menu_with(
        providers,
        status_labels,
        enabled_providers,
        false,
        bar_state,
    )
}

pub(crate) fn build_tray_menu_with(
    _providers: &[ProviderCatalogEntry],
    _status_labels: &[(String, String)],
    _enabled_providers: &HashSet<String>,
    _float_bar_enabled: bool,
    bar_state: BarState,
) -> Vec<TrayMenuEntry> {
    vec![
        TrayMenuEntry::item("toggle_detail", "Detail 열기"),
        TrayMenuEntry::item("toggle_bar_visibility", "Bar 표시 토글"),
        TrayMenuEntry::item("watchdog_retry", "Watchdog 재시도")
            .with_disabled(bar_state != BarState::Paused),
        TrayMenuEntry::item("open_settings", "설정 열기"),
        TrayMenuEntry::separator(),
        TrayMenuEntry::item("quit", "종료"),
    ]
}

impl TrayMenuEntry {
    fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

pub(crate) fn proof_menu_items(entries: &[TrayMenuEntry], menu_path: &str) -> Option<Vec<String>> {
    proof_menu_entries(entries, menu_path).map(|visible_entries| {
        visible_entries
            .iter()
            .filter(|entry| !entry.is_separator)
            .map(|entry| entry.label.clone())
            .collect()
    })
}

pub(crate) fn proof_menu_context_for_item(
    entries: &[TrayMenuEntry],
    item_id: &str,
) -> Option<(String, Vec<String>)> {
    proof_menu_context_for_item_inner(entries, item_id, "tray")
}

fn proof_menu_context_for_item_inner(
    entries: &[TrayMenuEntry],
    item_id: &str,
    menu_path: &str,
) -> Option<(String, Vec<String>)> {
    for entry in entries {
        if entry.is_separator {
            continue;
        }

        if entry.id.as_deref() == Some(item_id) {
            return proof_menu_items(entries, menu_path)
                .map(|items| (menu_path.to_string(), items));
        }

        if entry.children.is_empty() {
            continue;
        }

        let next_path = format!("{menu_path}/{}", entry.path_segment()?);
        if let Some(context) =
            proof_menu_context_for_item_inner(&entry.children, item_id, &next_path)
        {
            return Some(context);
        }
    }

    None
}

fn proof_menu_entries<'a>(
    entries: &'a [TrayMenuEntry],
    menu_path: &str,
) -> Option<&'a [TrayMenuEntry]> {
    let mut segments = menu_path.split('/');
    if segments.next()? != "tray" {
        return None;
    }

    let mut current = entries;
    for segment in segments {
        let submenu = current.iter().find(|entry| {
            !entry.is_separator
                && !entry.children.is_empty()
                && entry.path_segment().as_deref() == Some(segment)
        })?;
        current = &submenu.children;
    }

    Some(current)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_enabled() -> HashSet<String> {
        HashSet::new()
    }

    #[test]
    fn proof_menu_items_show_simplified_entries() {
        let items = proof_menu_items(
            &build_tray_menu(&[], &[], &empty_enabled(), BarState::Visible),
            "tray",
        )
        .unwrap();

        assert_eq!(
            items,
            vec!["Detail 열기", "Bar 표시 토글", "Watchdog 재시도", "설정 열기", "종료"]
        );
    }

    #[test]
    fn proof_menu_context_for_leaf_item_returns_parent_menu() {
        let (menu_path, items) = proof_menu_context_for_item(
            &build_tray_menu(&[], &[], &empty_enabled(), BarState::Visible),
            "quit",
        )
        .unwrap();

        assert_eq!(menu_path, "tray");
        assert!(items.iter().any(|item| item == "종료"));
    }

    #[test]
    fn watchdog_retry_is_disabled_until_bar_is_paused() {
        let visible_menu = build_tray_menu(&[], &[], &empty_enabled(), BarState::Visible);
        let visible_retry = visible_menu
            .iter()
            .find(|entry| entry.id.as_deref() == Some("watchdog_retry"))
            .expect("retry entry");
        assert!(visible_retry.disabled);

        let paused_menu = build_tray_menu(&[], &[], &empty_enabled(), BarState::Paused);
        let paused_retry = paused_menu
            .iter()
            .find(|entry| entry.id.as_deref() == Some("watchdog_retry"))
            .expect("retry entry");
        assert!(!paused_retry.disabled);
    }
}
