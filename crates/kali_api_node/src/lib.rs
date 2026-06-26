//! Node.js API compatibility surface for Kali runtime.
//!
//! This crate currently provides the first tranche of pure-Rust host-side helpers used by the
//! Phase-3 Node-compatibility work. The runtime still gates `--api node`, but the shared helper
//! layer is now concrete enough to be extended incrementally instead of remaining a stub.

mod assert;
pub use assert::*;

mod buffer;
pub use buffer::*;

mod child_process;
pub use child_process::*;

mod crypto;
pub use crypto::*;

mod events;
pub use events::*;

mod fs;
pub use fs::*;

mod http;
pub use http::*;

mod os;
pub use os::*;

mod path;
pub use path::*;

mod process;
pub use process::*;

mod runtime;
pub use runtime::*;

mod stream;
pub use stream::*;

mod url;
pub use url::*;

mod util;
pub use util::*;
