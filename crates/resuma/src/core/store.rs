//! Reactive store — structured reactive state for components.
//!
//! A `Store<T>` wraps a [`Signal<T>`] and represents a structured reactive
//! object. Mutations go through [`Store::update`] or field setters generated
//! by the `#[derive(Store)]` macro (future). The entire store serializes as
//! one JSON blob in the resumability payload.

use std::ops::Deref;

use serde::{Deserialize, Serialize};

use super::signal::{Signal, SignalId};

/// Reactive object state. Deep mutations require going through [`Store::update`]
/// or replacing the whole value with [`Store::set`].
#[derive(Clone)]
pub struct Store<T> {
    signal: Signal<T>,
}

impl<T> Store<T>
where
    T: Clone + Serialize + for<'de> Deserialize<'de> + 'static,
{
    pub fn new(initial: T) -> Self {
        Self { signal: Signal::new(initial) }
    }

    pub fn id(&self) -> SignalId {
        self.signal.id()
    }

    pub fn get(&self) -> T {
        self.signal.get()
    }

    pub fn peek(&self) -> T {
        self.signal.peek()
    }

    pub fn set(&self, value: T) {
        self.signal.set(value);
    }

    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut T),
    {
        self.signal.update(f);
    }

    /// Borrow the inner [`Signal`] for interpolation in `view!`.
    pub fn signal(&self) -> &Signal<T> {
        &self.signal
    }
}

impl<T> Deref for Store<T>
where
    T: Clone + Serialize + for<'de> Deserialize<'de> + 'static,
{
    type Target = Signal<T>;

    fn deref(&self) -> &Self::Target {
        &self.signal
    }
}

/// Create a reactive store — `let user = use_store(User { .. });`
pub fn use_store<T>(initial: T) -> Store<T>
where
    T: Clone + Serialize + for<'de> Deserialize<'de> + 'static,
{
    Store::new(initial)
}

/// Marker for values that must not cross the resumability boundary.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoSerialize<T>(pub Option<T>);

impl<T> NoSerialize<T> {
    pub fn new(value: T) -> Self {
        Self(Some(value))
    }

    pub fn take(&mut self) -> Option<T> {
        self.0.take()
    }
}

pub fn no_serialize<T>(value: T) -> NoSerialize<T> {
    NoSerialize::new(value)
}
