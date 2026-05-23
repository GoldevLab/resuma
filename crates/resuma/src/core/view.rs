//! The View tree — Resuma's resumable equivalent of a virtual DOM.
//!
//! A `View` is just data. It is built by components, walked by the SSR
//! renderer to emit HTML, and never re-executed on the client. Reactivity is
//! recovered by the runtime through `Dynamic` nodes that carry a signal id
//! plus a small JS expression describing how to project the signal value.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::handler::HandlerRef;
use super::signal::SignalId;

/// Top-level view returned from a component.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum View {
    Text(String),
    /// Reactive text/HTML node bound to a signal.
    Dynamic(Dynamic),
    Element(Element),
    Fragment(Fragment),
    Component(ComponentMarker),
    /// A self-contained interactive island. Renders its own SSR HTML and
    /// declares the chunk that will be lazy-loaded on first interaction.
    Island(Island),
    /// Content projection slot — resolved from parent slotted children.
    Slot(SlotView),
    /// A rendered chunk of raw HTML — used for trusted output and for nested
    /// pre-rendered components that have already been flattened.
    Raw(String),
    Empty,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Element {
    pub tag: String,
    pub attrs: Vec<Attr>,
    pub children: Vec<Child>,
    /// Optional explicit DOM id. Mostly used by tests/devtools.
    pub dom_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Fragment {
    pub children: Vec<Child>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMarker {
    pub name: String,
    pub view: Box<View>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotView {
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Island {
    /// Stable, build-time id of the island chunk on disk (e.g. `counter`).
    pub chunk_id: String,
    /// Per-instance id (each `<Counter />` on the page gets its own).
    pub instance_id: String,
    /// Signal ids belonging to this island. Used by the runtime to wire
    /// reactivity locally.
    pub signal_ids: Vec<SignalId>,
    /// Pre-rendered SSR HTML of the island content.
    pub view: Box<View>,
    /// Props serialized as JSON, passed to the island's resume() entry.
    pub props: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Child {
    View(View),
    /// Raw text — convenience to avoid wrapping every literal in `View::Text`.
    Text(String),
}

impl From<View> for Child {
    fn from(v: View) -> Self {
        Child::View(v)
    }
}
impl From<&str> for Child {
    fn from(s: &str) -> Self {
        Child::Text(s.to_string())
    }
}
impl From<String> for Child {
    fn from(s: String) -> Self {
        Child::Text(s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attr {
    pub name: String,
    pub value: AttrValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AttrValue {
    /// Plain string attribute.
    Static(String),
    /// Reactive attribute bound to a signal.
    Dynamic {
        signal: SignalId,
        format: Option<String>,
    },
    /// Event handler (`onClick`, `onInput`, …). Resolved at SSR time to a
    /// `HandlerRef` pointing at a JS chunk.
    Handler(HandlerRef),
    /// Boolean attribute that is omitted when false.
    Bool(bool),
    /// Declarative `preventDefault` for event handlers (async-safe).
    PreventDefault(String),
    /// Declarative `stopPropagation` for event handlers (async-safe).
    StopPropagation(String),
}

/// Reactive text/HTML node bound to a signal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dynamic {
    pub signal: SignalId,
    /// Optional format string. `{}` is replaced with the signal's value.
    pub format: Option<String>,
    /// JSON-serialized snapshot of the signal value at SSR time.
    pub snapshot: Value,
}

impl View {
    pub fn text(s: impl Into<String>) -> Self {
        View::Text(s.into())
    }
    pub fn raw(html: impl Into<String>) -> Self {
        View::Raw(html.into())
    }
    pub fn empty() -> Self {
        View::Empty
    }

    pub fn element(tag: impl Into<String>) -> ElementBuilder {
        ElementBuilder {
            element: Element {
                tag: tag.into(),
                ..Default::default()
            },
        }
    }

    pub fn fragment(children: Vec<Child>) -> Self {
        View::Fragment(Fragment { children })
    }

    pub fn slot(name: Option<String>) -> Self {
        View::Slot(SlotView { name })
    }
}

/// Ergonomic builder used by the `view!` macro expansion.
pub struct ElementBuilder {
    element: Element,
}

impl ElementBuilder {
    pub fn attr(mut self, name: impl Into<String>, value: AttrValue) -> Self {
        self.element.attrs.push(Attr {
            name: name.into(),
            value,
        });
        self
    }

    pub fn child(mut self, child: impl Into<Child>) -> Self {
        self.element.children.push(child.into());
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = Child>) -> Self {
        self.element.children.extend(children);
        self
    }

    pub fn dom_id(mut self, id: impl Into<String>) -> Self {
        self.element.dom_id = Some(id.into());
        self
    }

    pub fn build(self) -> View {
        View::Element(self.element)
    }
}
