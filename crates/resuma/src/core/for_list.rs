//! Reactive keyed `<For each={signal}>` — client diffing via `<resuma-for>`.

use serde::Serialize;
use serde_json::Value;

use super::effect::Computed;
use super::signal::{Signal, SignalId};
use super::view::{Child, ForItemView, ForView, View};

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
    let items = list
        .iter()
        .enumerate()
        .map(|(idx, item)| ForItemView {
            key: item_key(item, key_field, idx),
            children: render(item),
        })
        .collect();

    View::For(ForView {
        signal: each.list_id(),
        key_field: key_field.map(str::to_string),
        items,
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
}
