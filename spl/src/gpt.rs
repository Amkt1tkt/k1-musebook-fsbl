use heapless::{LinearMap, String};

use super::{
    Nvme, cpu,
    handoff::{
        DTB_BASE, DTB_PARTITION_NAME, DTB_SIZE, INITRAMFS_BASE, INITRAMFS_PARTITION_NAME,
        INITRAMFS_SIZE, KERNEL_BASE, KERNEL_PARTITION_NAME, KERNEL_SIZE, SBI_BASE,
        SBI_PARTITION_NAME, SBI_SIZE,
    },
};

pub struct Gpt {
    nvme: Nvme,
    partitions: LinearMap<String<72>, Partition, 16>,
}

impl Gpt {
    pub const GPT_HEADER_LBA: u64 = 1;
    pub const GPT_HEADER_SIGNATURE: [u8; 8] = *b"EFI PART";
    pub const ENTRY_BYTES: usize = 0x80;
    pub const LBA_BYTES: usize = Nvme::LBA_BYTES;
    pub const ENTRY_COUNT_PER_LBA: usize = Self::LBA_BYTES / Self::ENTRY_BYTES;

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

    pub fn load_all_partitions(&mut self) {
        log::info!("load all partitions");

        self.partitions
            .get(KERNEL_PARTITION_NAME)
            .unwrap_or_else(|| panic!("Kernel partition not found"))
            .load(&mut self.nvme, KERNEL_BASE, KERNEL_SIZE);

        self.partitions
            .get(SBI_PARTITION_NAME)
            .unwrap_or_else(|| panic!("SBI partition not found"))
            .load(&mut self.nvme, SBI_BASE, SBI_SIZE);

        self.partitions
            .get(DTB_PARTITION_NAME)
            .unwrap_or_else(|| panic!("DTB partition not found"))
            .load(&mut self.nvme, DTB_BASE, DTB_SIZE);

        self.partitions
            .get(INITRAMFS_PARTITION_NAME)
            .unwrap_or_else(|| panic!("Initramfs partition not found"))
            .load(&mut self.nvme, INITRAMFS_BASE, INITRAMFS_SIZE);
    }

    pub fn list_all_partitions(&self) {
        log::info!("partitions:");
        for (name, partition) in self.partitions.iter() {
            let size = partition.size_bytes;
            log::info!("    {name}: {size} bytes");
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Partition {
    pub start_lba: u64,
    pub size_bytes: u64,
}

impl Partition {
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

#[repr(C)]
struct GptEntryInfo {
    lba: u64,
    count: u32,
    size: u32,
}

impl GptEntryInfo {
    const OFFSET: usize = 0x48;
    fn from_gpt_header(gpt_header: &[u8]) -> &Self {
        unsafe { &*(gpt_header[Self::OFFSET..].as_ptr() as *const Self) }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct GptEntry {
    pub partition_type_guid: [u8; 16],
    pub unique_partition_guid: [u8; 16],
    pub starting_lba: u64,
    pub ending_lba: u64,
    pub attributes: u64,
    pub partition_name: [u16; 36],
}

impl GptEntry {
    fn from_chunk(chunk: &[u8]) -> [Self; Gpt::ENTRY_COUNT_PER_LBA] {
        unsafe { *(chunk.as_ptr() as *const [Self; Gpt::ENTRY_COUNT_PER_LBA]) }
    }
}
