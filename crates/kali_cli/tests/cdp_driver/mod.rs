//! Test-only Chrome DevTools Protocol driver for real-browser smoke coverage.
mod driver;
mod protocol;

pub(crate) use driver::chromium_available;
pub use driver::{CdpBrowser, CdpPageOutcome};
