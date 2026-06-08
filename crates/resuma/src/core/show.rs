//! Conditional rendering — Leptos-style `Show` without a separate macro system.

use super::signal::Signal;
use super::view::{Child, Fragment, ShowView, View};

/// Render `children` when `when` is true, otherwise `fallback` or nothing.
///
/// Evaluated once at SSR — use [`show_signal`] (via `<Show when={signal}>`) for
/// client-side toggling.
pub fn show(when: bool, children: Vec<Child>, fallback: Option<View>) -> View {
    if when {
        View::Fragment(Fragment { children })
    } else if let Some(fb) = fallback {
        fb
    } else {
        View::empty()
    }
}

/// Reactive show bound to a bool signal. Both branches are rendered in the DOM;
/// the client runtime toggles `hidden` on each branch.
pub fn show_signal(
    when: &Signal<bool>,
    inverted: bool,
    children: Vec<Child>,
    fallback: Option<View>,
) -> View {
    let raw = when.peek();
    let initial = if inverted { !raw } else { raw };
    View::Show(ShowView {
        signal: when.id(),
        inverted,
        initial,
        children,
        fallback: fallback.map(Box::new),
    })
}

#[cfg(test)]
mod tests {
    use super::super::context::{with_context, RenderContext, RenderMode};
    use super::super::signal;
    use super::super::view::{Child, View};
    use super::{show, show_signal};
    use crate::ssr::render_view;

    #[test]
    fn show_renders_children_when_true() {
        let v = show(true, vec![Child::Text("hi".into())], None);
        assert!(matches!(v, View::Fragment(_)));
    }

    #[test]
    fn show_renders_fallback_when_false() {
        let fb = View::text("no");
        let v = show(false, vec![], Some(fb));
        assert!(matches!(v, View::Text(s) if s == "no"));
    }

    #[test]
    fn show_signal_emits_resuma_show_marker() {
        let ctx = RenderContext::new(RenderMode::Ssr);
        let html = with_context(ctx, || {
            let on = signal(true);
            let v = show_signal(
                &on,
                false,
                vec![Child::Text("yes".into())],
                Some(View::text("no")),
            );
            render_view(&v)
        });
        assert!(html.contains("<resuma-show"));
        assert!(html.contains("data-r-show="));
        assert!(html.contains("data-r-show-if"));
        assert!(html.contains("yes"));
        assert!(html.contains("data-r-show-else"));
        assert!(html.contains("hidden"));
    }
}
