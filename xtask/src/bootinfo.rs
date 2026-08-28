//! pack the 80-byte NOR bootinfo image that BROM reads from offset 0
//!
//! Layout matches official `bootinfo_spinor.json` + `tools/build_binary_file.py`:
//! 0x40-byte header, IEEE CRC32 over that header, 12-byte pad. Total 0x50.

use std::{fs, path::Path};

use color_eyre::eyre::{Result, ensure};
use serde::Deserialize;

pub const BYTES: usize = 0x50;
const HEADER_BYTES: usize = 0x40;
const MAGIC: u32 = 0xB007_14F0;
const VERSION: u32 = 0x0001_0001;
const FLASH_TYPE: [u8; 4] = *b"NORF";
const NOR_SECTOR: u32 = 0x1000;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Bootinfo {
    pub spl0_offset: u32,
    pub spl1_offset: u32,
    pub spl_size_limit: u32,
    pub page_size: u32,
    pub block_size: u32,
    pub total_size: u32,
    pub partitiontable0_offset: u32,
    pub partitiontable1_offset: u32,
}

impl Default for Bootinfo {
    fn default() -> Self {
        Self {
            spl0_offset: 0x2_0000,
            spl1_offset: 0x7_0000,
            spl_size_limit: 0x3_6000,
            page_size: 256,
            block_size: 0x1_0000,
            total_size: 0x10_0000,
            partitiontable0_offset: 0,
            partitiontable1_offset: 0,
        }
    }
}

impl Bootinfo {
    pub fn load(path: &Path) -> Result<Self> {
        let info: Self = toml::from_str(&fs::read_to_string(path)?)?;
        info.validate()?;
        Ok(info)
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        ensure!(
            data.len() >= HEADER_BYTES + 4,
            "image is {} bytes, need at least {}",
            data.len(),
            HEADER_BYTES + 4
        );
        let magic = u32::from_le_bytes(data[0x00..0x04].try_into().unwrap());
        ensure!(
            magic == MAGIC,
            "invalid magic {magic:#010x}, expected {MAGIC:#010x}"
        );
        let version = u32::from_le_bytes(data[0x04..0x08].try_into().unwrap());
        ensure!(
            version == VERSION,
            "invalid version {version:#010x}, expected {VERSION:#010x}"
        );
        ensure!(data[0x08..0x0C] == FLASH_TYPE, "flash_type is not NORF");
        let crc_got = u32::from_le_bytes(data[0x40..0x44].try_into().unwrap());
        let crc_want = crc32fast::hash(&data[..HEADER_BYTES]);
        ensure!(
            crc_got == crc_want,
            "crc32 mismatch: image {crc_got:#010x}, computed {crc_want:#010x}"
        );
        Ok(Self {
            page_size: u32::from_le_bytes(data[0x10..0x14].try_into().unwrap()),
            block_size: u32::from_le_bytes(data[0x14..0x18].try_into().unwrap()),
            total_size: u32::from_le_bytes(data[0x18..0x1C].try_into().unwrap()),
            spl0_offset: u32::from_le_bytes(data[0x20..0x24].try_into().unwrap()),
            spl1_offset: u32::from_le_bytes(data[0x24..0x28].try_into().unwrap()),
            spl_size_limit: u32::from_le_bytes(data[0x28..0x2C].try_into().unwrap()),
            partitiontable0_offset: u32::from_le_bytes(data[0x2C..0x30].try_into().unwrap()),
            partitiontable1_offset: u32::from_le_bytes(data[0x30..0x34].try_into().unwrap()),
        })
    }

    pub fn to_toml(&self) -> String {
        format!(
            indoc::indoc!(
                "
                spl0_offset = {:#x}
                spl1_offset = {:#x}
                spl_size_limit = {:#x}

                page_size = {}
                block_size = {:#x}
                total_size = {:#x}
                partitiontable0_offset = {:#x}
                partitiontable1_offset = {:#x}
                "
            ),
            self.spl0_offset,
            self.spl1_offset,
            self.spl_size_limit,
            self.page_size,
            self.block_size,
            self.total_size,
            self.partitiontable0_offset,
            self.partitiontable1_offset,
        )
    }

    pub fn to_bytes(&self) -> [u8; BYTES] {
        let mut buf = [0u8; BYTES];
        buf[0x00..0x04].copy_from_slice(&MAGIC.to_le_bytes());
        buf[0x04..0x08].copy_from_slice(&VERSION.to_le_bytes());
        buf[0x08..0x0C].copy_from_slice(&FLASH_TYPE);
        buf[0x10..0x14].copy_from_slice(&self.page_size.to_le_bytes());
        buf[0x14..0x18].copy_from_slice(&self.block_size.to_le_bytes());
        buf[0x18..0x1C].copy_from_slice(&self.total_size.to_le_bytes());
        buf[0x20..0x24].copy_from_slice(&self.spl0_offset.to_le_bytes());
        buf[0x24..0x28].copy_from_slice(&self.spl1_offset.to_le_bytes());
        buf[0x28..0x2C].copy_from_slice(&self.spl_size_limit.to_le_bytes());
        buf[0x2C..0x30].copy_from_slice(&self.partitiontable0_offset.to_le_bytes());
        buf[0x30..0x34].copy_from_slice(&self.partitiontable1_offset.to_le_bytes());
        let crc = crc32fast::hash(&buf[..HEADER_BYTES]);
        buf[0x40..0x44].copy_from_slice(&crc.to_le_bytes());
        buf
    }

    pub fn brief(&self) -> String {
        let crc = u32::from_le_bytes(self.to_bytes()[0x40..0x44].try_into().unwrap());
        format!(
            "spl0={:#x} spl1={:#x} limit={:#x} crc32={crc:#010x}",
            self.spl0_offset, self.spl1_offset, self.spl_size_limit
        )
    }

    fn validate(&self) -> Result<()> {
        ensure!(self.spl_size_limit > 0, "spl_size_limit must be > 0");
        ensure!(
            self.spl0_offset >= NOR_SECTOR,
            "spl0 offset {:#x} overlaps bootinfo (NOR 0..{NOR_SECTOR:#x})",
            self.spl0_offset
        );
        if self.spl1_offset != 0 {
            ensure!(
                self.spl1_offset >= NOR_SECTOR,
                "spl1 offset {:#x} overlaps bootinfo (NOR 0..{NOR_SECTOR:#x})",
                self.spl1_offset
            );
            let a = self.spl0_offset;
            let b = self.spl0_offset.saturating_add(self.spl_size_limit);
            let c = self.spl1_offset;
            let d = self.spl1_offset.saturating_add(self.spl_size_limit);
            ensure!(
                a >= d || c >= b,
                "spl0 [{a:#x}..{b:#x}) overlaps spl1 [{c:#x}..{d:#x})"
            );
        }
        Ok(())
    }
}
