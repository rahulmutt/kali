//! Web API compatibility surface for Kali runtime.

#[cfg(test)]
pub(crate) use serde_json::Value;
#[cfg(test)]
pub(crate) use std::sync::Arc;

mod base64;
pub use base64::*;

mod crypto;
pub use crypto::*;

mod events;
pub use events::*;

mod fetch;
pub use fetch::*;

mod file;
pub use file::*;

mod indexeddb;
pub use indexeddb::*;

mod navigator;
pub use navigator::*;

mod storage;
pub use storage::*;

mod streams;
pub use streams::*;

mod threads;
pub use threads::*;

mod url;
pub use url::*;

mod util;
pub use util::*;

mod websocket;
pub use websocket::*;

mod worker;
pub use worker::*;
