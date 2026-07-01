//! Test-only Chrome DevTools Protocol driver for real-browser smoke coverage.
mod driver;
mod protocol;

pub use driver::{CdpBrowser, CdpConsoleLine, CdpPageOutcome};
