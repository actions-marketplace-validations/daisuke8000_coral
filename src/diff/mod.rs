//! Diff module for comparing two GraphModels.

mod core;
mod markdown;

pub use self::core::{Change, DiffItems, DiffNode, DiffReport, ModifiedItem};

#[cfg(test)]
mod tests;
