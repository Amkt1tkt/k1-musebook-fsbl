use k1_musebook_spl::nvme::Nvme;
use postcard_rpc::header::VarHeader;

use super::{
    ByteBuf, CHUNK_BYTES, Context, FlashServerError, NorChunk, NorRange, NvmeChunk, NvmeRange,
    ReadResult, VERSION, WriteResult, nor,
};

pub fn ping(_ctx: &mut Context, _hdr: VarHeader, _req: ()) -> u32 {
    VERSION
}

pub fn nor_erase(_ctx: &mut Context, _hdr: VarHeader, req: NorRange) -> WriteResult {
    nor::erase(req.offset, req.len)
}

pub fn nor_write(_ctx: &mut Context, _hdr: VarHeader, req: NorChunk) -> WriteResult {
    nor::write(req.offset, req.data)
}

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

pub fn nvme_write(ctx: &mut Context, _hdr: VarHeader, req: NvmeChunk) -> WriteResult {
    if req.data.is_empty() || !req.data.len().is_multiple_of(Nvme::LBA_BYTES) {
        return Err(FlashServerError::Args);
    }
    ctx.nvme.write(req.lba, req.data);
    Ok(())
}

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
