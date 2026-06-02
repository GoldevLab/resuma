mod book;
mod gracias;
mod index;

pub use book::BookPage;
pub use gracias::GraciasPage;
pub use index::IndexPage;

include!("_registry.rs");
