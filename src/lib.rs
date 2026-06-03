pub mod app;
pub mod error;
pub mod excel;
// pub mod ui;
pub mod pipeline;
pub use error::{Error, Result};

#[cfg(test)]
pub mod test;
