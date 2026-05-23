//! Component trait & helpers.

use serde::Serialize;

use crate::signal::Signal;
use crate::store::Store;
use crate::view::{Child, Dynamic, View};

/// Trait implemented by every renderable unit. The `#[component]` macro
/// generates an implementation pointing at the component's render fn.
///
/// Components receive their props by value; reactive state lives in signals
/// captured inside the body. Components are *only* invoked on the server.
pub trait Component {
    type Props: Clone + Send + Sync + 'static;

    fn name() -> &'static str;
    fn render(props: Self::Props) -> View;
}

/// Anything that can be turned into a `View` for child interpolation in the
/// `view!{}` macro. The trait takes `&self` so interpolating `{count}` does
/// not move `count` out of the surrounding scope — important because the
/// same signal is typically referenced from multiple event handlers.
pub trait IntoView {
    fn into_view(&self) -> View;
}

impl IntoView for View {
    fn into_view(&self) -> View { self.clone() }
}

impl IntoView for str {
    fn into_view(&self) -> View { View::text(self) }
}

impl IntoView for String {
    fn into_view(&self) -> View { View::text(self.clone()) }
}

impl IntoView for &str {
    fn into_view(&self) -> View { View::text(*self) }
}

macro_rules! impl_into_view_display {
    ($($t:ty),*) => {
        $(
            impl IntoView for $t {
                fn into_view(&self) -> View { View::text(self.to_string()) }
            }
        )*
    }
}

impl_into_view_display!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64, bool, char);

impl<T: IntoView> IntoView for Option<T> {
    fn into_view(&self) -> View {
        match self {
            Some(v) => v.into_view(),
            None => View::empty(),
        }
    }
}

impl<T: IntoView> IntoView for Vec<T> {
    fn into_view(&self) -> View {
        let children: Vec<Child> = self
            .iter()
            .map(|v| Child::View(v.into_view()))
            .collect();
        View::fragment(children)
    }
}

/// `Signal<T>` interpolated inside `view!{ {count} }` becomes a reactive
/// `<resuma-dyn>` node bound to the signal id.
impl<T: Clone + Serialize + 'static> IntoView for Signal<T> {
    fn into_view(&self) -> View {
        let snapshot = serde_json::to_value(self.peek()).unwrap_or(serde_json::Value::Null);
        View::Dynamic(Dynamic {
            signal: self.id(),
            format: None,
            snapshot,
        })
    }
}

impl<T: Clone + Serialize + 'static> IntoView for &Signal<T> {
    fn into_view(&self) -> View {
        (*self).into_view()
    }
}

impl<T: Clone + Serialize + for<'de> serde::Deserialize<'de> + 'static> IntoView for Store<T> {
    fn into_view(&self) -> View {
        self.signal().into_view()
    }
}

impl<T: Clone + Serialize + for<'de> serde::Deserialize<'de> + 'static> IntoView for &Store<T> {
    fn into_view(&self) -> View {
        self.signal().into_view()
    }
}
