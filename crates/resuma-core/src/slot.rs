//! Content projection — `<Slot />` and `slot="name"` child routing.
//!
//! Children passed to a component can declare a target slot via the `slot`
//! attribute: `<h2 slot="header">Title</h2>`. Inside the component body,
//! `<Slot name="header" />` resolves to those children.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use crate::view::{Child, View};

thread_local! {
    static SLOT_STACK: RefCell<Vec<Rc<SlottedChildren>>> = const { RefCell::new(Vec::new()) };
}

/// A child node tagged with an optional slot name (`None` = default slot).
#[derive(Debug, Clone)]
pub struct SlottedChild {
    pub slot: Option<String>,
    pub child: Child,
}

/// Resolved slot map for the currently rendering component.
#[derive(Debug, Clone, Default)]
pub struct SlottedChildren {
    default: Vec<Child>,
    named: BTreeMap<String, Vec<Child>>,
}

impl SlottedChildren {
    pub fn from_vec(items: Vec<SlottedChild>) -> Self {
        let mut out = Self::default();
        for item in items {
            match &item.slot {
                None => out.default.push(item.child),
                Some(s) if s.is_empty() => out.default.push(item.child),
                Some(name) => out.named.entry(name.clone()).or_default().push(item.child),
            }
        }
        out
    }

    pub fn resolve(&self, name: Option<&str>) -> View {
        let children: Vec<Child> = match name {
            None | Some("") => self.default.clone(),
            Some(n) => self.named.get(n).cloned().unwrap_or_default(),
        };
        if children.is_empty() {
            View::empty()
        } else if children.len() == 1 {
            match &children[0] {
                Child::View(v) => v.clone(),
                Child::Text(t) => View::text(t.clone()),
            }
        } else {
            View::fragment(children)
        }
    }
}

/// Push slotted children for the duration of a component render.
pub fn push_slots(items: Vec<SlottedChild>) -> SlotGuard {
    let map = Rc::new(SlottedChildren::from_vec(items));
    SLOT_STACK.with(|stack| stack.borrow_mut().push(map));
    SlotGuard
}

pub struct SlotGuard;

impl Drop for SlotGuard {
    fn drop(&mut self) {
        SLOT_STACK.with(|stack| {
            stack.borrow_mut().pop();
        });
    }
}

/// Wrap `page` as the default slot while rendering a layout component.
pub fn with_default_slot(content: View, f: impl FnOnce() -> View) -> View {
    let _guard = push_slots(vec![SlottedChild {
        slot: None,
        child: Child::View(content),
    }]);
    f()
}

/// Resolve a slot from the innermost component on the stack.
pub fn resolve_slot(name: Option<&str>) -> View {
    SLOT_STACK.with(|stack| {
        stack
            .borrow()
            .last()
            .map(|s| s.resolve(name))
            .unwrap_or(View::empty())
    })
}
