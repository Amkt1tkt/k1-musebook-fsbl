//! Read the GPT from NVMe LBA 1 and load named partitions into DDR.
//!
//! Entries are indexed by UTF-16 partition name. After each load, `cache::clean`
//! so later harts and SBI can fetch the image from cache.

use heapless::{LinearMap, String};

use super::{GPT_PARTITIONS, Nvme, cpu};

/// NVMe-backed GPT with a name → range map (up to 16 entries).
pub struct Gpt {
    nvme: Nvme,
    partitions: LinearMap<String<72>, Partition, 16>,
}

impl Gpt {
    /// Protective MBR is LBA 0; the GPT header is always LBA 1.
    pub const GPT_HEADER_LBA: u64 = 1;
    /// `EFI PART` signature at the start of the GPT header.
    pub const GPT_HEADER_SIGNATURE: [u8; 8] = *b"EFI PART";
    /// Bytes per GPT entry (UEFI spec).
    pub const ENTRY_BYTES: usize = 0x80;
    /// NVMe logical block size used for GPT I/O.
    pub const LBA_BYTES: usize = Nvme::LBA_BYTES;
    /// GPT entries packed in one LBA.
    pub const ENTRY_COUNT_PER_LBA: usize = Self::LBA_BYTES / Self::ENTRY_BYTES;

    /// Read the header and entries, indexing used partitions by UTF-16 name.
    pub fn parse(mut nvme: Nvme) -> Self {
        let mut gpt_header = [0u8; Self::LBA_BYTES];
        nvme.read(Self::GPT_HEADER_LBA, &mut gpt_header);
        if gpt_header[0..8] != Self::GPT_HEADER_SIGNATURE {
            panic!("GPT header not found");
        }
        let entry_info = GptEntryInfo::from_gpt_header(&gpt_header);
        if entry_info.size != Self::ENTRY_BYTES as u32 {
            panic!("GPT entry size is not correct");
        }

        let partitions = (0..entry_info.count as u64)
            .step_by(Self::ENTRY_COUNT_PER_LBA)
            .map(|index| index / Self::ENTRY_COUNT_PER_LBA as u64 + entry_info.lba)
            .map(|lba| {
                let mut chunk = [0u8; Self::LBA_BYTES];
                nvme.read(lba, &mut chunk);
                chunk
            })
            .flat_map(|chunk| GptEntry::from_chunk(&chunk))
            .filter(|entry| entry.partition_type_guid != [0u8; 16])
            .map(|entry| {
                let name = entry
                    .partition_name
                    .split(|&c| c == 0)
                    .next()
                    .unwrap_or_default();
                let size = entry.ending_lba - entry.starting_lba + 1;
                (
                    String::from_utf16(name).unwrap_or_default(),
                    Partition {
                        start_lba: entry.starting_lba,
                        size_bytes: size * Self::LBA_BYTES as u64,
                    },
                )
            })
            .collect();
        Self { nvme, partitions }
    }

    /// Load every `GPT_PARTITIONS` entry into its DDR window (then `cache::clean`).
    pub fn load_all_partitions(&mut self) {
        log::info!("load all partitions");
        for part in GPT_PARTITIONS {
            self.partitions
                .get(part.name)
                .unwrap_or_else(|| panic!("{} partition not found", part.name))
                .load(&mut self.nvme, part.load_base, part.load_max);
        }
    }

    /// Log each discovered partition name and size.
    pub fn list_all_partitions(&self) {
        log::info!("partitions:");
        for (name, partition) in self.partitions.iter() {
            let size = partition.size_bytes;
            log::info!("    {name}: {size} bytes");
        }
    }
}

/// On-disk GPT range (start LBA + size) used when loading into DDR.
#[derive(Debug, Clone, Copy)]
pub struct Partition {
    /// First LBA of the partition.
    pub start_lba: u64,
    /// Size in bytes (LBA count × 512).
    pub size_bytes: u64,
}

impl Partition {
    /// Read into `[mem_start, …)` (capped by `mem_max_size`) and clean the D-cache.
    fn load(self, nvme: &mut Nvme, mem_start: u64, mem_max_size: u64) {
        let read_bytes = self.size_bytes.next_multiple_of(Nvme::LBA_BYTES as u64);
        if read_bytes > mem_max_size {
            panic!("Partition size is too large");
        }
        let dst =
            unsafe { core::slice::from_raw_parts_mut(mem_start as *mut u8, read_bytes as usize) };
        nvme.read(self.start_lba, dst);
        cpu::cache::clean(mem_start as usize, read_bytes as usize);
    }
}

/// GPT header fields at offset 0x48: entry LBA, count, and entry size.
#[repr(C)]
struct GptEntryInfo {
    lba: u64,
    count: u32,
    size: u32,
}

impl GptEntryInfo {
    const OFFSET: usize = 0x48;
    /// Reinterpret the header bytes at offset 0x48 as `GptEntryInfo`.
    fn from_gpt_header(gpt_header: &[u8]) -> &Self {
        unsafe { &*(gpt_header[Self::OFFSET..].as_ptr() as *const Self) }
    }
}

/// One GPT partition entry (128 bytes, UTF-16 name).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct GptEntry {
    /// Partition type GUID; all-zero means unused.
    pub partition_type_guid: [u8; 16],
    /// Unique partition GUID.
    pub unique_partition_guid: [u8; 16],
    /// First LBA.
    pub starting_lba: u64,
    /// Last LBA (inclusive).
    pub ending_lba: u64,
    /// GPT attribute bits.
    pub attributes: u64,
    /// UTF-16LE name, NUL-padded.
    pub partition_name: [u16; 36],
}

impl GptEntry {
    /// Reinterpret one LBA as `ENTRY_COUNT_PER_LBA` entries.
    fn from_chunk(chunk: &[u8]) -> [Self; Gpt::ENTRY_COUNT_PER_LBA] {
        unsafe { *(chunk.as_ptr() as *const [Self; Gpt::ENTRY_COUNT_PER_LBA]) }
    }
}
