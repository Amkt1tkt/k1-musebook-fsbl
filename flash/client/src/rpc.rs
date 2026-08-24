use std::path::Path;

use k1_musebook_flash_server::protocol::{
    CHUNK_BYTES, FlashServerError, NorChunk, NorEraseEndpoint, NorRange, NorReadEndpoint,
    NorWriteEndpoint, NvmeChunk, NvmeRange, NvmeReadEndpoint, NvmeWriteEndpoint, PingEndpoint,
    VERSION,
};
use k1_musebook_spl::gpt::Gpt;
use postcard_rpc::{
    host_client::{HostClient, HostErr},
    standard_icd::WireError,
};
use tokio::fs;

use super::{RpcReady, Usb};

const CHUNK_LBA_COUNT: u64 = CHUNK_BYTES as u64 / Gpt::LBA_BYTES as u64;

pub struct FlashClient(HostClient<WireError>);

impl FlashClient {
    pub async fn connect(usb: Usb<RpcReady>) -> Result<Self, HostErr<WireError>> {
        println!("connecting to flash server ...");
        let client = Self(usb.client());
        let resp = client.ping().await?;
        println!("server connected, version={resp:#010x} (expected {VERSION:#010x})");
        Ok(client)
    }

    pub async fn ping(&self) -> Result<u32, HostErr<WireError>> {
        self.0.send_resp::<PingEndpoint>(&()).await
    }

    pub async fn nor_read(&self, offset: u32, len: u32) -> Result<Vec<u8>, FlashClientError> {
        println!("nor read {len} bytes from {offset:#x} ...");
        let mut out = Vec::with_capacity(len as usize);

        let full_chunk_count = len / CHUNK_BYTES as u32;
        let tail_chunk_bytes = len % CHUNK_BYTES as u32;
        let tail_chunk = tail_chunk_bytes
            .gt(&0)
            .then_some((full_chunk_count, tail_chunk_bytes));
        let chunks = (0..full_chunk_count)
            .into_iter()
            .map(|index| (index, CHUNK_BYTES as u32))
            .chain(tail_chunk);

        let start = tokio::time::Instant::now();

        for (index, chunk_len) in chunks {
            let chunk_offset = offset + index * CHUNK_BYTES as u32;
            let resp = self
                .0
                .send_resp::<NorReadEndpoint>(&NorRange {
                    offset: chunk_offset,
                    len: chunk_len,
                })
                .await?;
            let done = ((index + 1) * CHUNK_BYTES as u32).min(len);
            let progress = done as f64 / len as f64 * 100.0;
            let elapsed = start.elapsed().as_secs_f64();
            println!(
                "nor read {chunk_len} bytes from {chunk_offset:#x} {progress:.2}% {elapsed:.2}s"
            );
            out.extend_from_slice(&resp?.data);
        }

        println!("nor read complete");
        Ok(out)
    }

    pub async fn nor_erase(&self, offset: u32, len: u32) -> Result<(), FlashClientError> {
        self.0
            .send_resp::<NorEraseEndpoint>(&NorRange { offset, len })
            .await??;
        Ok(())
    }

    pub async fn nor_write_file(&self, offset: u32, path: &Path) -> Result<(), NorWriteFileError> {
        let file = fs::read(path).await?;
        if file.is_empty() {
            return Ok(());
        }
        let file_path = path.display();
        let file_size = file.len() as u32;
        let end = offset
            .checked_add(file_size)
            .ok_or(NorWriteFileError::OffsetOverflow)?;
        let erase_off = offset & !0xFFF;
        let erase_end = (end + 0xFFF) & !0xFFF;
        let erase_len = erase_end - erase_off;

        println!("nor erase {erase_len} bytes from {erase_off:#x} to {erase_end:#x} ...");
        self.nor_erase(erase_off, erase_len).await?;

        println!("nor write {file_size} bytes from {file_path} to {offset:#x} ...");
        self.nor_write_bytes(offset, &file).await?;

        println!("nor write complete");
        Ok(())
    }

    pub async fn nor_write_bytes(
        &self,
        offset: u32,
        data: &[u8],
    ) -> Result<(), NorWriteBytesError> {
        let start = tokio::time::Instant::now();
        for (index, chunk) in data.chunks(CHUNK_BYTES).enumerate() {
            let chunk_offset = offset + index as u32 * CHUNK_BYTES as u32;
            self.0
                .send_resp::<NorWriteEndpoint>(&NorChunk {
                    offset: chunk_offset,
                    data: chunk,
                })
                .await??;
            let done = ((index + 1) * CHUNK_BYTES).min(data.len());
            let progress = done as f64 / data.len() as f64 * 100.0;
            let elapsed = start.elapsed().as_secs_f64();
            println!("wrote {CHUNK_BYTES} bytes to {chunk_offset:#x} {progress:.2}% {elapsed:.2}s");
        }
        Ok(())
    }

    pub async fn nvme_read(&self, lba: u64, len: u32) -> Result<Vec<u8>, NvmeReadBytesError> {
        self.nvme_read_bytes(lba, len as u64).await
    }

    pub async fn nvme_read_bytes(&self, lba: u64, len: u64) -> Result<Vec<u8>, NvmeReadBytesError> {
        println!("nvme read {len} bytes from {lba:#x} ...");
        let mut out = Vec::with_capacity(len as usize);

        let full_chunk_count = len / CHUNK_BYTES as u64;
        let tail_chunk_bytes = len % CHUNK_BYTES as u64;
        let tail_chunk = tail_chunk_bytes
            .gt(&0)
            .then_some((full_chunk_count, tail_chunk_bytes));
        let chunks = (0..full_chunk_count)
            .into_iter()
            .map(|index| (index, CHUNK_BYTES as u64))
            .chain(tail_chunk);

        let start = tokio::time::Instant::now();

        for (index, chunk_len) in chunks {
            let chunk_lba = lba + index * CHUNK_LBA_COUNT;
            let resp = self
                .0
                .send_resp::<NvmeReadEndpoint>(&NvmeRange {
                    lba: chunk_lba,
                    len: chunk_len as u32,
                })
                .await?;
            let done = ((index + 1) * CHUNK_BYTES as u64).min(len);
            let progress = done as f64 / len as f64 * 100.0;
            let elapsed = start.elapsed().as_secs_f64();
            println!(
                "nvme read {chunk_len} bytes from {chunk_lba:#x} {progress:.2}% {elapsed:.2}s"
            );
            out.extend_from_slice(&resp?.data);
        }

        println!("nvme read complete");
        Ok(out)
    }

    pub async fn nvme_write_file(&self, lba: u64, path: &Path) -> Result<(), NvmeWriteFileError> {
        let file = fs::read(path).await?;
        let file_path = path.display();
        let file_size = file.len() as u64;

        println!("nvme write {file_size} bytes from {file_path} to {lba:#x} ...");
        self.nvme_write_bytes(lba, &file).await?;

        println!("nvme write complete");
        Ok(())
    }

    pub async fn nvme_write_bytes(&self, lba: u64, data: &[u8]) -> Result<(), NvmeWriteBytesError> {
        let mut vec = data.to_vec();
        let pad = (512 - vec.len() % 512) % 512;
        vec.extend(std::iter::repeat_n(0xFFu8, pad));

        let start = tokio::time::Instant::now();

        for (index, chunk) in vec.chunks(CHUNK_BYTES).enumerate() {
            let chunk_lba = lba + index as u64 * CHUNK_LBA_COUNT;
            self.0
                .send_resp::<NvmeWriteEndpoint>(&NvmeChunk {
                    lba: chunk_lba,
                    data: chunk,
                })
                .await??;
            let done = ((index + 1) * CHUNK_BYTES).min(vec.len());
            let progress = done as f64 / vec.len() as f64 * 100.0;
            let elapsed = start.elapsed().as_secs_f64();
            println!("wrote {CHUNK_BYTES} bytes to {chunk_lba:#x} {progress:.2}% {elapsed:.2}s");
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FlashClientError {
    #[error("wire error: {0}")]
    Wire(#[from] HostErr<WireError>),
    #[error("operate failed: {0}")]
    Operate(#[from] FlashServerError),
}

#[derive(Debug, thiserror::Error)]
pub enum NorWriteBytesError {
    #[error("wire error: {0}")]
    Wire(#[from] HostErr<WireError>),
    #[error("operate failed: {0}")]
    Operate(#[from] FlashServerError),
}

#[derive(Debug, thiserror::Error)]
pub enum NorWriteFileError {
    #[error("read file failed: {0}")]
    ReadFile(#[from] std::io::Error),
    #[error("offset overflow")]
    OffsetOverflow,
    #[error("erase failed: {0}")]
    EraseFailed(#[from] FlashClientError),
    #[error("write bytes failed: {0}")]
    WriteBytesFailed(#[from] NorWriteBytesError),
}

#[derive(Debug, thiserror::Error)]
pub enum NvmeWriteBytesError {
    #[error("read file failed: {0}")]
    ReadFile(#[from] std::io::Error),
    #[error("wire error: {0}")]
    Wire(#[from] HostErr<WireError>),
    #[error("operate failed: {0}")]
    Operate(#[from] FlashServerError),
}

#[derive(Debug, thiserror::Error)]
pub enum NvmeWriteFileError {
    #[error("read file failed: {0}")]
    ReadFile(#[from] std::io::Error),
    #[error("write bytes failed: {0}")]
    WriteBytesFailed(#[from] NvmeWriteBytesError),
}

#[derive(Debug, thiserror::Error)]
pub enum NvmeReadBytesError {
    #[error("read file failed: {0}")]
    ReadFile(#[from] std::io::Error),
    #[error("wire error: {0}")]
    Wire(#[from] HostErr<WireError>),
    #[error("operate failed: {0}")]
    Operate(#[from] FlashServerError),
}
