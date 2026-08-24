use std::path::PathBuf;

use k1_musebook_spl::gpt::Gpt;
use tap::prelude::*;
use tokio::fs;

use super::{FlashClient, NvmeReadBytesError, NvmeWriteBytesError};

const PMBR_LBA_COUNT: u64 = 1;
const GPT_HEADER_LBA_COUNT: u64 = 1;
const ENTRY_LBA: u64 = Gpt::GPT_HEADER_LBA + GPT_HEADER_LBA_COUNT;
const ENTRY_COUNT: u64 = 128;
const ENTRY_LBA_COUNT: u64 = ENTRY_COUNT / Gpt::ENTRY_COUNT_PER_LBA as u64;
const TOTAL_ENTRIES_BYTES: u64 = ENTRY_COUNT * Gpt::ENTRY_BYTES as u64;

const HEAD_OCCUPY_LBA_COUNT: u64 = PMBR_LBA_COUNT + GPT_HEADER_LBA_COUNT + ENTRY_LBA_COUNT;
const FIRST_USABLE_LBA: u64 = HEAD_OCCUPY_LBA_COUNT;

const ALTERNATE_GPT_HEADER_LBA_COUNT: u64 = 1;
const TAIL_OCCUPY_LBA_COUNT: u64 = ALTERNATE_GPT_HEADER_LBA_COUNT + ENTRY_LBA_COUNT;

const LAYOUT: &[(&str, u64, u64)] = &[
    ("sbi", 2048, 1024),          // 512 KiB
    ("kernel", 4096, 24576),      // 12 MiB
    ("dtb", 28672, 512),          // 256 KiB
    ("initramfs", 32768, 131072), // 64 MiB
    ("rootfs", 163840, 0),        // remaining
];

impl FlashClient {
    pub async fn gpt_list(&self) -> Result<Vec<Partition>, GptParseError> {
        let partitions = self.gpt_parse().await?;
        println!(
            "{:<16} {:>10} {:>10} {:>12}",
            "NAME", "START", "END", "SIZE"
        );
        for partition in &partitions {
            println!(
                "{:<16} {:>10} {:>10} {:>12}",
                partition.name,
                partition.start_lba,
                partition.end_lba,
                partition.size_bytes()
            );
        }
        Ok(partitions)
    }

    pub async fn gpt_init(&self, disk_lba_count: Option<u64>) -> Result<(), GptInitError> {
        println!("gpt partition table init start ...");
        let alternate_gpt_header_lba = self
            .gpt_read_alternate_header_lba()
            .await?
            .or_else(|| disk_lba_count.map(|n| n.saturating_sub(1)))
            .ok_or(GptInitError::UnknownDiskLbaCount)?;
        let disk_lba_count = alternate_gpt_header_lba + ALTERNATE_GPT_HEADER_LBA_COUNT;
        let last_usable = alternate_gpt_header_lba
            .checked_sub(TAIL_OCCUPY_LBA_COUNT)
            .ok_or(GptInitError::DiskTooSmall)?;
        let partitions = resolve_layout(last_usable)?;
        println!("gpt init partitions: {partitions:#?}");

        let disk_guid = random_guid();
        let entries = build_entries(&partitions);
        let entries_crc = crc32fast::hash(&entries);
        let mut primary =
            Vec::with_capacity((HEAD_OCCUPY_LBA_COUNT * Gpt::LBA_BYTES as u64) as usize);
        primary.extend_from_slice(&build_pmbr(disk_lba_count));
        primary.extend_from_slice(&build_header(
            Gpt::GPT_HEADER_LBA,
            alternate_gpt_header_lba,
            FIRST_USABLE_LBA,
            last_usable,
            ENTRY_LBA,
            disk_guid,
            entries_crc,
        ));
        primary.extend_from_slice(&entries);
        println!("writing primary GPT to lba 0 ...");
        self.nvme_write_bytes(0, &primary).await?;

        let alternate_entry_lba = alternate_gpt_header_lba - ENTRY_LBA_COUNT;
        let mut alternate = entries;
        alternate.extend_from_slice(&build_header(
            alternate_gpt_header_lba,
            Gpt::GPT_HEADER_LBA,
            FIRST_USABLE_LBA,
            last_usable,
            alternate_entry_lba,
            disk_guid,
            entries_crc,
        ));
        println!("writing alternate GPT to lba {alternate_entry_lba} ...");
        self.nvme_write_bytes(alternate_entry_lba, &alternate)
            .await?;

        println!("gpt init complete");
        Ok(())
    }

    pub async fn gpt_flash(&self, parts: &[(String, PathBuf)]) -> Result<(), GptFlashError> {
        let partitions = self.gpt_parse().await?;

        let mut jobs = Vec::with_capacity(parts.len());
        for (name, path) in parts {
            let partition = partitions
                .iter()
                .find(|p| &p.name == name)
                .ok_or_else(|| GptFlashError::NotFound(name.clone()))?;
            let file_size = fs::metadata(path).await?.len();
            if file_size > partition.size_bytes() {
                return Err(GptFlashError::FileTooLarge(
                    file_size,
                    partition.size_bytes(),
                ));
            }
            jobs.push((name, path, partition.start_lba));
        }

        for (name, path, start_lba) in jobs {
            let file = fs::read(path).await?;
            let file_path = path.display();
            let file_size = file.len();
            println!("flash {name}: {file_path} ({file_size}B) to lba {start_lba} ...");
            self.nvme_write_bytes(start_lba, &file).await?;
        }

        println!("gpt flash complete");
        Ok(())
    }

    async fn gpt_parse(&self) -> Result<Vec<Partition>, GptParseError> {
        let gpt_header = self
            .nvme_read_bytes(Gpt::GPT_HEADER_LBA, Gpt::LBA_BYTES as u64)
            .await?;
        if gpt_header[0..8] != Gpt::GPT_HEADER_SIGNATURE {
            return Err(GptParseError::NotGptHeader);
        }

        let entry_lba = u64::from_le_bytes(gpt_header[72..80].try_into().unwrap());
        let entry_count = u32::from_le_bytes(gpt_header[80..84].try_into().unwrap());
        let entry_size = u32::from_le_bytes(gpt_header[84..88].try_into().unwrap());

        let all_entries_bytes =
            (entry_count * entry_size).next_multiple_of(Gpt::LBA_BYTES as u32) as u64;

        self.nvme_read_bytes(entry_lba, all_entries_bytes)
            .await?
            .chunks(Gpt::ENTRY_BYTES)
            .filter(|raw_entry| raw_entry[0..16] != [0u8; 16])
            .map(|raw_entry| {
                let start = u64::from_le_bytes(raw_entry[32..40].try_into().unwrap());
                let end = u64::from_le_bytes(raw_entry[40..48].try_into().unwrap());
                let name = raw_entry[56..128]
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|char| u16::from_le_bytes([char[0], char[1]]))
                    .take_while(|&char| char != 0)
                    .collect::<Vec<u16>>()
                    .pipe(|chars| String::from_utf16_lossy(&chars));
                Partition {
                    name,
                    start_lba: start,
                    end_lba: end,
                }
            })
            .collect::<Vec<Partition>>()
            .pipe(Ok)
    }

    async fn gpt_read_alternate_header_lba(&self) -> Result<Option<u64>, NvmeReadBytesError> {
        let gpt_header = self
            .nvme_read_bytes(Gpt::GPT_HEADER_LBA, Gpt::LBA_BYTES as u64)
            .await?;
        if gpt_header[0..8] != Gpt::GPT_HEADER_SIGNATURE {
            return Ok(None);
        }
        Ok(Some(u64::from_le_bytes(
            gpt_header[32..40].try_into().unwrap(),
        )))
    }
}

#[derive(Debug, Clone)]
pub struct Partition {
    pub name: String,
    pub start_lba: u64,
    pub end_lba: u64,
}

impl Partition {
    pub fn size_bytes(&self) -> u64 {
        (self.end_lba - self.start_lba + 1) * Gpt::LBA_BYTES as u64
    }
}

fn resolve_layout(last_usable: u64) -> Result<Vec<(String, u64, u64)>, GptInitError> {
    LAYOUT
        .iter()
        .map(|(name, start, size)| {
            (
                name.to_string(),
                *start,
                if *size == 0 {
                    last_usable.saturating_sub(*start) + 1
                } else {
                    *size
                },
            )
        })
        .map(|(name, start, size)| {
            if start + size - 1 > last_usable {
                Err(GptInitError::PartitionOutOfRange(name, last_usable))
            } else {
                Ok((name, start, size))
            }
        })
        .collect::<Result<Vec<(String, u64, u64)>, GptInitError>>()
}

const TYPE_LINUX: [u8; 16] = [
    0xaf, 0x3d, 0xc6, 0x0f, 0x83, 0x84, 0x72, 0x47, 0x8e, 0x79, 0x3d, 0x69, 0xd8, 0x47, 0x7d, 0xe4,
];

fn build_entries(partitions: &[(String, u64, u64)]) -> Vec<u8> {
    let mut entries = vec![0u8; TOTAL_ENTRIES_BYTES as usize];
    for (index, (name, start, size)) in partitions.iter().enumerate() {
        let offset = index * Gpt::ENTRY_BYTES;
        entries[offset + 00..offset + 16].copy_from_slice(&TYPE_LINUX);
        entries[offset + 16..offset + 32].copy_from_slice(&random_guid());
        entries[offset + 32..offset + 40].copy_from_slice(&start.to_le_bytes());
        entries[offset + 40..offset + 48].copy_from_slice(&(start + size - 1).to_le_bytes());
        let utf16 = name
            .encode_utf16()
            .flat_map(|char| char.to_le_bytes())
            .collect::<Vec<u8>>();
        let name_bytes = utf16.len().min(72);
        entries[offset + 56..offset + 56 + name_bytes].copy_from_slice(&utf16[..name_bytes]);
    }
    entries
}

fn build_header(
    current_lba: u64,
    alternate_lba: u64,
    first_usable: u64,
    last_usable: u64,
    entry_lba: u64,
    disk_guid: [u8; 16],
    entries_crc: u32,
) -> [u8; 512] {
    let mut gpt_header = [0u8; 512];
    gpt_header[00..08].copy_from_slice(&Gpt::GPT_HEADER_SIGNATURE);
    gpt_header[08..12].copy_from_slice(&0x0001_0000u32.to_le_bytes());
    gpt_header[12..16].copy_from_slice(&92u32.to_le_bytes());
    gpt_header[24..32].copy_from_slice(&current_lba.to_le_bytes());
    gpt_header[32..40].copy_from_slice(&alternate_lba.to_le_bytes());
    gpt_header[40..48].copy_from_slice(&first_usable.to_le_bytes());
    gpt_header[48..56].copy_from_slice(&last_usable.to_le_bytes());
    gpt_header[56..72].copy_from_slice(&disk_guid);
    gpt_header[72..80].copy_from_slice(&entry_lba.to_le_bytes());
    gpt_header[80..84].copy_from_slice(&(ENTRY_COUNT as u32).to_le_bytes());
    gpt_header[84..88].copy_from_slice(&(Gpt::ENTRY_BYTES as u32).to_le_bytes());
    gpt_header[88..92].copy_from_slice(&entries_crc.to_le_bytes());
    let crc = crc32fast::hash(&gpt_header[0..92]);
    gpt_header[16..20].copy_from_slice(&crc.to_le_bytes());
    gpt_header
}

fn build_pmbr(disk_lba_count: u64) -> [u8; 512] {
    let mut mbr = [0u8; 512];
    let off = 446;
    mbr[off + 1..off + 4].copy_from_slice(&[0x00, 0x02, 0x00]);
    mbr[off + 4] = 0xEE;
    mbr[off + 5..off + 8].copy_from_slice(&[0xFF, 0xFF, 0xFF]);
    mbr[off + 8..off + 12].copy_from_slice(&1u32.to_le_bytes());
    let lba = disk_lba_count.saturating_sub(1).min(u32::MAX as u64) as u32;
    mbr[off + 12..off + 16].copy_from_slice(&lba.to_le_bytes());
    mbr[510] = 0x55;
    mbr[511] = 0xAA;
    mbr
}

fn random_guid() -> [u8; 16] {
    rand::random::<[u8; 16]>()
}

#[derive(thiserror::Error, Debug)]
pub enum GptParseError {
    #[error("LBA1 is not a GPT header")]
    NotGptHeader,
    #[error("failed to read GPT entries: {0}")]
    NvmeReadError(#[from] NvmeReadBytesError),
}

#[derive(thiserror::Error, Debug)]
pub enum GptFlashError {
    #[error("failed to parse GPT: {0}")]
    GptParseError(#[from] GptParseError),
    #[error("failed to write GPT entries: {0}")]
    NvmeWriteError(#[from] NvmeWriteBytesError),
    #[error("failed to read file: {0}")]
    ReadFileError(#[from] std::io::Error),
    #[error("partition `{0}` not found")]
    NotFound(String),
    #[error("file too large: {0} exceeds partition {1}")]
    FileTooLarge(u64, u64),
}

#[derive(thiserror::Error, Debug)]
pub enum GptInitError {
    #[error("failed to read alternate GPT header: {0}")]
    NvmeReadError(#[from] NvmeReadBytesError),
    #[error("unknown disk LBA count")]
    UnknownDiskLbaCount,
    #[error("disk too small for GPT")]
    DiskTooSmall,
    #[error("partition {0} out of range: {1}")]
    PartitionOutOfRange(String, u64),
    #[error("failed to write GPT: {0}")]
    NvmeWriteError(#[from] NvmeWriteBytesError),
}
