//! Reactive keyed `<For each={signal}>` — client diffing via `<resuma-for>`.

use std::collections::HashSet;

use serde::Serialize;
use serde_json::Value;

use super::effect::Computed;
use super::signal::{Signal, SignalId};
use super::view::{Child, ForItemView, ForView, View, VirtualListOpts};

/// Anything that exposes a list signal id and SSR snapshot.
pub trait ListSignal<T> {
    fn list_id(&self) -> SignalId;
    fn list_peek(&self) -> Vec<T>;
}

impl<T: Clone + Serialize + Send + Sync + 'static> ListSignal<T> for Signal<Vec<T>> {
    fn list_id(&self) -> SignalId {
        self.id()
    }
    fn list_peek(&self) -> Vec<T> {
        self.peek()
    }
}

impl<T: Clone + Serialize + Send + Sync + 'static> ListSignal<T> for Computed<Vec<T>> {
    fn list_id(&self) -> SignalId {
        self.id()
    }
    fn list_peek(&self) -> Vec<T> {
        self.peek()
    }
}

/// Reactive keyed list — SSR renders current items; the client reconciles by key.
pub fn for_signal<T, S, F>(each: &S, key_field: Option<&str>, mut render: F) -> View
where
    T: Clone + Serialize + Send + Sync + 'static,
    S: ListSignal<T>,
    F: FnMut(&T) -> Vec<Child>,
{
    let list = each.list_peek();
    let mut seen_keys = HashSet::new();
    let items = list
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            let mut key = item_key(item, key_field, idx);
            if !seen_keys.insert(key.clone()) {
                tracing::warn!(
                    key = %key,
                    index = idx,
                    "duplicate <For> key — appending index suffix"
                );
                key = format!("{key}:{idx}");
            }
            ForItemView {
                key,
                children: render(item),
            }
        })
        .collect();

    View::For(ForView {
        signal: each.list_id(),
        key_field: key_field.map(str::to_string),
        items,
        virtual_list: None,
    })
}

/// Like [`for_signal`] but SSR-renders a window and lets the client recycle rows.
pub fn for_signal_virtual<T, S, F>(
    each: &S,
    key_field: Option<&str>,
    item_height: u32,
    overscan: u32,
    mut render: F,
) -> View
where
    T: Clone + Serialize + Send + Sync + 'static,
    S: ListSignal<T>,
    F: FnMut(&T) -> Vec<Child>,
{
    let list = each.list_peek();
    let total = list.len();
    let height = item_height.max(1);
    let over = if overscan == 0 { 6 } else { overscan };
    let window = ((600 / height) + over * 2).max(8) as usize;
    let mut seen_keys = HashSet::new();
    let items = list
        .iter()
        .take(window)
        .enumerate()
        .map(|(idx, item)| {
            let mut key = item_key(item, key_field, idx);
            if !seen_keys.insert(key.clone()) {
                tracing::warn!(
                    key = %key,
                    index = idx,
                    "duplicate <For> key — appending index suffix"
                );
                key = format!("{key}:{idx}");
            }
            ForItemView {
                key,
                children: render(item),
            }
        })
        .collect();

    View::For(ForView {
        signal: each.list_id(),
        key_field: key_field.map(str::to_string),
        items,
        virtual_list: Some(VirtualListOpts {
            item_height: height,
            overscan: over,
            total,
        }),
    })
}

fn item_key<T: Serialize>(item: &T, key_field: Option<&str>, idx: usize) -> String {
    if let Some(field) = key_field {
        if let Ok(v) = serde_json::to_value(item) {
            if let Some(k) = v.get(field) {
                return json_key(k);
            }
        }
    }
    idx.to_string()
}

fn json_key(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::context::{with_context, RenderContext, RenderMode};
    use crate::ssr::render_view;

    #[derive(Clone, Serialize)]
    struct Row {
        id: u64,
        title: String,
    }

    #[test]
    fn for_signal_emits_resuma_for_marker() {
        let ctx = RenderContext::new(RenderMode::Ssr);
        let html = with_context(ctx, || {
            let rows = Signal::new(vec![Row {
                id: 1,
                title: "a".into(),
            }]);
            let v = for_signal(&rows, Some("id"), |r| vec![Child::Text(r.title.clone())]);
            render_view(&v)
        });
        assert!(html.contains("<resuma-for"));
        assert!(html.contains("data-r-for-key=\"1\""));
    }

    #[test]
    fn for_signal_disambiguates_duplicate_keys() {
        let ctx = RenderContext::new(RenderMode::Ssr);
        let html = with_context(ctx, || {
            let rows = Signal::new(vec![
                Row {
                    id: 1,
                    title: "a".into(),
                },
                Row {
                    id: 1,
                    title: "b".into(),
                },
            ]);
            let v = for_signal(&rows, Some("id"), |r| vec![Child::Text(r.title.clone())]);
            render_view(&v)
        });
        assert!(html.contains("data-r-for-key=\"1\""));
        assert!(html.contains("data-r-for-key=\"1:1\""));
    }

    #[test]
    fn for_signal_virtual_emits_window_marker() {
        let ctx = RenderContext::new(RenderMode::Ssr);
        let html = with_context(ctx, || {
            let rows = Signal::new(
                (0..100)
                    .map(|i| Row {
                        id: i,
                        title: format!("r{i}"),
                    })
                    .collect(),
            );
            let v = for_signal_virtual(&rows, Some("id"), 40, 2, |r| {
                vec![Child::Text(r.title.clone())]
            });
            render_view(&v)
        });
        assert!(html.contains("data-r-virtual"));
        assert!(html.contains("data-r-virtual-total=\"100\""));
        assert!(html.contains("data-r-item-height=\"40\""));
        assert!(
            !html.contains("data-r-for-key=\"99\""),
            "SSR should not paint every row: {html}"
        );
    }
}
