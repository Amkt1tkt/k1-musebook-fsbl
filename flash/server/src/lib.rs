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
