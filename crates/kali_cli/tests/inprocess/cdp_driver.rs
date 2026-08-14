//! Shim so `inprocess/browser_harness_cdp_in_page_trap_propagates.rs` (unchanged
//! since its move from `tests/`) can resolve its unqualified `mod cdp_driver;`
//! from its new directory. Re-paths into the shared driver also used by the
//! standalone `browser_cdp_smoke` target rather than duplicating it.
#[path = "../cdp_driver/driver.rs"]
mod driver;
#[path = "../cdp_driver/protocol.rs"]
mod protocol;

pub(crate) use driver::chromium_available;
pub use driver::{CdpBrowser, CdpPageOutcome};
