//! # Library to read and write protocol buffers data
//!
//! ## Version 3 is alpha
//!
//! Currently developed branch of rust-protobuf is 3. It has the same spirit as version 2,
//! but contains numerous improvements like:
//! * runtime reflection for mutability, not just for access
//! * protobuf text format and JSON parsing (which rely on reflection)
//! * dynamic message support: work with protobuf data without generating code from schema
//! * lite runtime codegen option now produces correct code without reflection support
//!
//! Stable version of rust-protobuf will be supported until version 3 released.
//!
//! [Tracking issue for version 3](https://github.com/stepancheg/rust-protobuf/issues/518).
//!
//! ## Features
//!
//! This crate has one feature, which is `with-bytes`.
//!
//! `with-bytes` enables `protobuf` crate support for
//! [`bytes` crate](https://github.com/tokio-rs/bytes):
//! when parsing bytes or strings from `bytes::Bytes`,
//! `protobuf` will be able to reference the input instead of allocating subarrays.
//!
//! Note, codegen also need to be instructed to generate `Bytes` or `Chars` for
//! `bytes` or `string` protobuf types instead of default `Vec<u8>` or `String`,
//! just enabling option on this crate is not enough.
//!
//! See `Customize` struct in [`protobuf-codegen` crate](https://docs.rs/protobuf/%3E=3.0.0-alpha).
//!
//! ## Accompanying crates
//!
//! * [`protobuf-codegen`](https://docs.rs/protobuf-codegen/%3E=3.0.0-alpha)
//!   can be used to rust code from `.proto` crates.
//! * [`protoc-bin-vendored`](https://docs.rs/protoc-bin-vendored/%3E=3.0.0-alpha)
//!   contains `protoc` command packed into the crate.
//! * [`protobuf-parse`](https://docs.rs/protobuf-parse/%3E=3.0.0-alpha) contains
//!   `.proto` file parser. Rarely need to be used directly,
//!   but can be used for mechanical processing of `.proto` files.

#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

pub use crate::coded_input_stream::CodedInputStream;
pub use crate::coded_output_stream::CodedOutputStream;
pub use crate::enum_full::EnumFull;
pub use crate::enum_or_unknown::EnumOrUnknown;
pub use crate::enums::Enum;
pub use crate::message::Message;
pub use crate::message_dyn::MessageDyn;
pub use crate::message_field::MessageField;
pub use crate::message_full::MessageFull;
pub use crate::oneof::Oneof;
pub use crate::unknown::UnknownFields;
pub use crate::unknown::UnknownFieldsIter;
pub use crate::unknown::UnknownValue;
pub use crate::unknown::UnknownValueRef;
pub use crate::unknown::UnknownValues;
pub use crate::unknown::UnknownValuesIter;
pub(crate) mod wire_format;
#[cfg(feature = "bytes")]
pub use crate::chars::Chars;
pub use crate::error::Error;
pub use crate::error::Result;

// generated
pub mod descriptor;
pub mod plugin;
pub mod rustproto;

mod coded_input_stream;
mod coded_output_stream;
mod enum_full;
mod enum_or_unknown;
mod enums;
mod error;
pub mod ext;
pub mod json;
mod lazy;
mod message;
mod message_dyn;
mod message_field;
mod message_full;
mod oneof;
pub mod reflect;
pub mod rt;
pub mod text_format;
pub mod well_known_types;
mod well_known_types_util;

// used by test
#[cfg(test)]
#[path = "../../test-crates/protobuf-test-common/src/hex.rs"]
mod hex;

mod cached_size;
mod chars;
mod unknown;
mod varint;
mod zigzag;

mod misc;

mod buf_read_iter;
mod buf_read_or_reader;

#[cfg(feature = "bytes")]
mod bytes_iterator_reader;

// TODO: does not work: https://github.com/rust-lang/rust/issues/67295
#[cfg(doctest)]
mod doctest_pb;

/// This symbol is in generated `version.rs`, include here for IDE
#[cfg(never)]
pub const VERSION: &str = "";
/// This symbol is in generated `version.rs`, include here for IDE
#[cfg(never)]
#[doc(hidden)]
pub const VERSION_IDENT: &str = "";
include!(concat!(env!("OUT_DIR"), "/version.rs"));
