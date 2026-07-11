//! Reactive `<Match value={signal}>` with `<When is={…}>` branches.

use serde::Serialize;

use super::effect::Computed;
use super::signal::{Signal, SignalId};
use super::view::{Child, Fragment, MatchCase, MatchView, View};

/// Anything that exposes a discriminant signal id and SSR snapshot.
pub trait MatchSignal<T> {
    fn match_id(&self) -> SignalId;
    fn match_peek(&self) -> T;
}

impl<T: Clone + Serialize + Send + Sync + 'static> MatchSignal<T> for Signal<T> {
    fn match_id(&self) -> SignalId {
        self.id()
    }
    fn match_peek(&self) -> T {
        self.peek()
    }
}

impl<T: Clone + Serialize + Send + Sync + 'static> MatchSignal<T> for Computed<T> {
    fn match_id(&self) -> SignalId {
        self.id()
    }
    fn match_peek(&self) -> T {
        self.peek()
    }
}

pub fn match_static(
    current: String,
    cases: Vec<(String, Vec<Child>)>,
    default: Option<Vec<Child>>,
) -> View {
    let cases: Vec<MatchCase> = cases
        .into_iter()
        .map(|(when, children)| MatchCase { when, children })
        .collect();
    for case in &cases {
        if case.when == current {
            return View::Fragment(Fragment {
                children: case.children.clone(),
            });
        }
    }
    if let Some(children) = default {
        View::Fragment(Fragment { children })
    } else {
        View::empty()
    }
}

/// Reactive multi-branch match — all branches SSR; client toggles visibility.
pub fn match_signal<T, S>(
    value: &S,
    cases: Vec<(String, Vec<Child>)>,
    default: Option<Vec<Child>>,
) -> View
where
    T: Clone + Serialize + Send + Sync + 'static,
    S: MatchSignal<T>,
{
    let current = match_value_string(&value.match_peek());
    let cases = cases
        .into_iter()
        .map(|(when, children)| MatchCase { when, children })
        .collect();

    View::Match(MatchView {
        signal: value.match_id(),
        initial: current,
        cases,
        default,
    })
}

pub fn match_value_string<T: Serialize>(value: &T) -> String {
    serde_json::to_value(value)
        .ok()
        .map(|v| match &v {
            serde_json::Value::String(s) => s.clone(),
            other => other.to_string(),
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::context::{with_context, RenderContext, RenderMode};
    use crate::ssr::render_view;

    #[test]
    fn match_value_string_matches_json_to_string() {
        assert_eq!(match_value_string(&Option::<String>::None), "null");
        assert_eq!(match_value_string(&42_i32), "42");
        assert_eq!(match_value_string(&true), "true");
        assert_eq!(
            match_value_string(&serde_json::json!({"a": 1})),
            r#"{"a":1}"#
        );
        assert_eq!(match_value_string(&serde_json::json!(["x"])), r#"["x"]"#);
    }

    #[test]
    fn match_signal_emits_resuma_match_marker() {
        let ctx = RenderContext::new(RenderMode::Ssr);
        let html = with_context(ctx, || {
            let mode = Signal::new("active".to_string());
            let v = match_signal(
                &mode,
                vec![
                    ("active".into(), vec![Child::Text("on".into())]),
                    ("done".into(), vec![Child::Text("off".into())]),
                ],
                None,
            );
            render_view(&v)
        });
        assert!(html.contains("<resuma-match"));
        assert!(html.contains("data-r-match-case"));
    }
}
