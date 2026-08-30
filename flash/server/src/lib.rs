//! On-board `no_std` flash-server: QSPI NOR, reused BROM USB, and postcard-rpc.
//!
//! The host flash-client talks USB to this crate; this crate reuses SPL drivers
//! from `k1-musebook-spl`.

#![no_std]

pub mod nor;
pub mod protocol;
pub mod rpc;
pub mod usb;

use self::protocol::{
    ByteBuf, CHUNK_BYTES, ENDPOINT_LIST, FlashServerError, NorChunk, NorEraseEndpoint, NorRange,
    NorReadEndpoint, NorWriteEndpoint, NvmeChunk, NvmeRange, NvmeReadEndpoint, NvmeWriteEndpoint,
    PingEndpoint, ReadResult, TOPICS_IN_LIST, TOPICS_OUT_LIST, VERSION, WriteResult,
};
