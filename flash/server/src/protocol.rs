//! postcard-rpc ICD shared by the host client and the on-board flash-server.
//!
//! `VERSION` is `0x0001_0000`. Payloads are chunked at 1 MiB. Endpoints cover
//! ping plus NOR/NVMe erase, write, and read over `NorRange`/`NorChunk` and
//! `NvmeRange`/`NvmeChunk`.

use heapless::Vec;
use postcard_rpc::{TopicDirection, endpoints, topics};
use postcard_schema::Schema;
use serde::{Deserialize, Serialize};

/// Maximum NOR/NVMe payload size per RPC (1 MiB).
pub const CHUNK_BYTES: usize = 1024 * 1024;

/// ICD version returned by `ping` (`0x0001_0000`).
pub const VERSION: u32 = 0x0001_0000;

/// NOR/NVMe write or erase result.
pub type WriteResult = Result<(), FlashServerError>;
/// NOR/NVMe read result.
pub type ReadResult = Result<ByteBuf, FlashServerError>;

/// Server-side failure: bad arguments or hardware timeout/error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Schema, thiserror::Error)]
pub enum FlashServerError {
    /// Offset, length, or alignment is invalid.
    #[error("invalid arguments")]
    Args,
    /// Controller timed out or reported a hardware fault.
    #[error("hardware error")]
    Hardware,
}

/// NOR byte range (`offset` + `len`).
#[derive(Debug, Clone, Serialize, Deserialize, Schema)]
pub struct NorRange {
    /// Byte offset into the QSPI AHB window.
    pub offset: u32,
    /// Length in bytes.
    pub len: u32,
}

/// NOR write payload (`offset` + `data`).
#[derive(Debug, Clone, Serialize, Deserialize, Schema)]
pub struct NorChunk<'a> {
    /// Byte offset into the QSPI AHB window.
    pub offset: u32,
    /// Bytes to program.
    pub data: &'a [u8],
}

/// NVMe read range (`lba` + byte `len`).
#[derive(Debug, Clone, Serialize, Deserialize, Schema)]
pub struct NvmeRange {
    /// Starting logical block address.
    pub lba: u64,
    /// Length in bytes (must be a multiple of 512).
    pub len: u32,
}

/// NVMe write payload (`lba` + `data`).
#[derive(Debug, Clone, Serialize, Deserialize, Schema)]
pub struct NvmeChunk<'a> {
    /// Starting logical block address.
    pub lba: u64,
    /// Bytes to write (must be a multiple of 512).
    pub data: &'a [u8],
}

/// Heapless buffer holding at most [`CHUNK_BYTES`] of read data.
#[derive(Debug, Clone, Serialize, Deserialize, Schema)]
pub struct ByteBuf {
    /// Read payload.
    pub data: Vec<u8, CHUNK_BYTES>,
}

endpoints! {
    list = ENDPOINT_LIST;
    omit_std = true;
    | EndpointTy        | RequestTy     | ResponseTy  | Path         |
    | ----------------- | ------------- | ----------- | ------------ |
    | PingEndpoint      | ()            | u32         | "ping"       |
    | NorEraseEndpoint  | NorRange      | WriteResult | "nor/erase"  |
    | NorWriteEndpoint  | NorChunk<'a>  | WriteResult | "nor/write"  |
    | NorReadEndpoint   | NorRange      | ReadResult  | "nor/read"   |
    | NvmeWriteEndpoint | NvmeChunk<'a> | WriteResult | "nvme/write" |
    | NvmeReadEndpoint  | NvmeRange     | ReadResult  | "nvme/read"  |
}

topics! {
    list = TOPICS_OUT_LIST;
    direction = TopicDirection::ToClient;
    omit_std = true;
    | TopicTy | MessageTy | Path |
    | ------- | --------- | ---- |
}

topics! {
    list = TOPICS_IN_LIST;
    direction = TopicDirection::ToServer;
    omit_std = true;
    | TopicTy | MessageTy | Path |
    | ------- | --------- | ---- |
}
