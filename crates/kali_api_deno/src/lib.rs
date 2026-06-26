//! Deno API compatibility surface for Kali runtime.
//!
//! This crate provides the Deno-oriented host-support layer that sits on top of the shared Web
//! baseline. It keeps the Phase-1 standalone surface focused on deterministic file/env/permission
//! views without inventing a browser/runtime shim or a mutable process model.

pub use kali_api_web::{
    atob, btoa, crypto, fetch, fill_random_values, local_storage, navigator, parse_url,
    performance_now, random_uuid, resolve_url, session_storage, structured_clone, text_decode,
    text_encode, AbortController, AbortSignal, Base64Error, Blob, BroadcastChannel, Crypto,
    CustomEvent, Event, EventTarget, File, FileReader, FileReaderState, FormData, FormDataEntry,
    FormDataValue, Headers, IndexedDB, IndexedDb, Navigator, ReadableStream, Request, Response,
    Storage, TransformStream, URLSearchParams, WebSocket, WebSocketReadyState, Worker,
    WritableStream, URL,
};

mod args;
pub use args::*;

mod command;
pub use command::*;

mod env;
pub use env::*;

mod fs;
pub use fs::*;

mod net;
pub use net::*;

mod path; // internal — no glob re-export

mod permissions;
pub use permissions::*;

mod runtime;
pub use runtime::*;

/// Initialize the Deno API compatibility surface.
pub fn deno_api_init() {
    kali_api_web::web_api_init();
}

#[cfg(test)]
#[path = "reexport_tests.rs"]
mod reexport_tests;
