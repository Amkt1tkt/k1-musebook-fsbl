use heapless::Vec;
use postcard_rpc::{TopicDirection, endpoints, topics};
use postcard_schema::Schema;
use serde::{Deserialize, Serialize};

pub const CHUNK_BYTES: usize = 1024 * 1024;

pub const VERSION: u32 = 0x0001_0000;

pub type WriteResult = Result<(), FlashServerError>;
pub type ReadResult = Result<ByteBuf, FlashServerError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Schema, thiserror::Error)]
pub enum FlashServerError {
    #[error("invalid arguments")]
    Args,
    #[error("hardware error")]
    Hardware,
}

#[derive(Debug, Clone, Serialize, Deserialize, Schema)]
pub struct NorRange {
    pub offset: u32,
    pub len: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Schema)]
pub struct NorChunk<'a> {
    pub offset: u32,
    pub data: &'a [u8],
}

#[derive(Debug, Clone, Serialize, Deserialize, Schema)]
pub struct NvmeRange {
    pub lba: u64,
    pub len: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Schema)]
pub struct NvmeChunk<'a> {
    pub lba: u64,
    pub data: &'a [u8],
}

#[derive(Debug, Clone, Serialize, Deserialize, Schema)]
pub struct ByteBuf {
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
