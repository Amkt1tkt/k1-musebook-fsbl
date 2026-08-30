//! postcard-rpc endpoint handlers that forward to `nor::*` and `Nvme::read` / `Nvme::write`.

use k1_musebook_spl::nvme::Nvme;
use postcard_rpc::header::VarHeader;

use super::{
    ByteBuf, CHUNK_BYTES, Context, FlashServerError, NorChunk, NorRange, NvmeChunk, NvmeRange,
    ReadResult, VERSION, WriteResult, nor,
};

/// Return the ICD [`VERSION`].
pub fn ping(_ctx: &mut Context, _hdr: VarHeader, _req: ()) -> u32 {
    VERSION
}

/// Erase a NOR range via [`nor::erase`].
pub fn nor_erase(_ctx: &mut Context, _hdr: VarHeader, req: NorRange) -> WriteResult {
    nor::erase(req.offset, req.len)
}

/// Program NOR bytes via [`nor::write`].
pub fn nor_write(_ctx: &mut Context, _hdr: VarHeader, req: NorChunk) -> WriteResult {
    nor::write(req.offset, req.data)
}

/// Read a NOR range into a [`ByteBuf`] (1..=[`CHUNK_BYTES`]).
pub fn nor_read(_ctx: &mut Context, _hdr: VarHeader, req: NorRange) -> ReadResult {
    let len = req.len as usize;
    if len == 0 || len > CHUNK_BYTES {
        return Err(FlashServerError::Args);
    }
    let mut data = heapless::Vec::new();
    if data.resize(len, 0).is_err() {
        return Err(FlashServerError::Args);
    }
    nor::read(req.offset, &mut data)?;
    Ok(ByteBuf { data })
}

/// Write NVMe bytes; `data.len()` must be a non-empty multiple of 512.
pub fn nvme_write(ctx: &mut Context, _hdr: VarHeader, req: NvmeChunk) -> WriteResult {
    if req.data.is_empty() || !req.data.len().is_multiple_of(Nvme::LBA_BYTES) {
        return Err(FlashServerError::Args);
    }
    ctx.nvme.write(req.lba, req.data);
    Ok(())
}

/// Read NVMe bytes; `len` must be a non-empty 512-multiple up to [`CHUNK_BYTES`].
pub fn nvme_read(ctx: &mut Context, _hdr: VarHeader, req: NvmeRange) -> ReadResult {
    let len = req.len as usize;
    if len == 0 || len > CHUNK_BYTES || !len.is_multiple_of(Nvme::LBA_BYTES) {
        return Err(FlashServerError::Args);
    }
    let mut data = heapless::Vec::new();
    if data.resize(len, 0).is_err() {
        return Err(FlashServerError::Args);
    }
    ctx.nvme.read(req.lba, &mut data);
    ReadResult::Ok(ByteBuf { data })
}
