use core::time::Duration;

use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};

use super::{
    ACQ_BASE, ACQ_SIZE, ADMIN_QID, ASQ_BASE, ASQ_SIZE, AcqBase, Aqa, AsqBase, Cap, Config, IO_QID,
    IOCQ_BASE, IOCQ_SIZE, IOSQ_BASE, IOSQ_SIZE, MMIO, NVME_CTRL, NVME_DOORBELL_BASE, READ_DMA_BASE,
    Raw, Status, cpu, time,
};

pub struct Nvme {
    cid: u16,
    dstrd: u8,
    asq_tail: u32,
    acq_head: u32,
    acq_phase: u8,
    iosq_tail: u32,
    iocq_head: u32,
    iocq_phase: u8,
}

impl Nvme {
    pub const LBA_BYTES: usize = 512;
    const CHUNK_LBAS: usize = 8;
    const CHUNK_BYTES: usize = Self::CHUNK_LBAS * Self::LBA_BYTES;
    const NSID: u32 = 1;

    pub fn open() -> Self {
        log::info!("nvme init");
        init_controller();
        let dstrd = NVME_CTRL.cap.read(Cap::DSTRD);
        let mut nvme = Self {
            cid: 0,
            dstrd: dstrd as u8,
            asq_tail: 0,
            acq_head: 0,
            acq_phase: 1,
            iosq_tail: 0,
            iocq_head: 0,
            iocq_phase: 1,
        };
        nvme.create_io_cq();
        nvme.create_io_sq();
        nvme
    }

    pub fn read(&mut self, lba: u64, dst: &mut [u8]) {
        dst.chunks_mut(Self::CHUNK_BYTES)
            .enumerate()
            .map(|(index, chunk)| (lba + index as u64 * Self::CHUNK_LBAS as u64, chunk))
            .for_each(|(lba, chunk)| {
                self.read_chunk(lba, chunk);
            });
    }

    pub fn write(&mut self, lba: u64, src: &[u8]) {
        src.chunks(Self::CHUNK_BYTES)
            .enumerate()
            .map(|(index, chunk)| (lba + index as u64 * Self::CHUNK_LBAS as u64, chunk))
            .for_each(|(lba, chunk)| {
                self.write_chunk(lba, chunk);
            });
    }

    fn read_chunk(&mut self, lba: u64, dst: &mut [u8]) {
        let sqe = Sqe {
            cdw0: SqeCw0 {
                opcode: IoOpcode::Read as u8,
                psdt_fuse: 0,
                cid: self.alloc_cid(),
            },
            nsid: Self::NSID,
            prp1: READ_DMA_BASE as u64,
            prp2: READ_DMA_BASE as u64 + 0x1000,
            cdw10: lba as u32,
            cdw11: (lba >> 32) as u32,
            cdw12: (dst.len() / Self::LBA_BYTES - 1) as u32,
            ..Default::default()
        };
        self.send_io_sqe(sqe);
        cpu::cache::inval(READ_DMA_BASE as usize, dst.len());
        unsafe {
            core::ptr::copy_nonoverlapping(
                READ_DMA_BASE as usize as *const u8,
                dst.as_mut_ptr(),
                dst.len(),
            );
        }
    }

    fn write_chunk(&mut self, lba: u64, src: &[u8]) {
        unsafe {
            core::ptr::copy_nonoverlapping(
                src.as_ptr(),
                READ_DMA_BASE as usize as *mut u8,
                src.len(),
            );
        }
        cpu::cache::clean(READ_DMA_BASE as usize, src.len());
        let sqe = Sqe {
            cdw0: SqeCw0 {
                opcode: IoOpcode::Write as u8,
                psdt_fuse: 0,
                cid: self.alloc_cid(),
            },
            nsid: Self::NSID,
            prp1: READ_DMA_BASE as u64,
            prp2: READ_DMA_BASE as u64 + 0x1000,
            cdw10: lba as u32,
            cdw11: (lba >> 32) as u32,
            cdw12: (src.len() / Self::LBA_BYTES - 1) as u32,
            ..Default::default()
        };
        self.send_io_sqe(sqe);
    }

    fn create_io_cq(&mut self) {
        let sqe = Sqe {
            cdw0: SqeCw0 {
                opcode: AdminOpcode::CreateIoCq as u8,
                psdt_fuse: 0,
                cid: self.alloc_cid(),
            },
            nsid: 0,
            prp1: IOCQ_BASE as u64,
            cdw10: ((IOCQ_SIZE - 1) << 16) | IO_QID,
            cdw11: 0x0000_0001,
            ..Default::default()
        };
        self.send_admin_sqe(sqe);
    }

    fn create_io_sq(&mut self) {
        let sqe = Sqe {
            cdw0: SqeCw0 {
                opcode: AdminOpcode::CreateIoSq as u8,
                psdt_fuse: 0,
                cid: self.alloc_cid(),
            },
            nsid: 0,
            prp1: IOSQ_BASE as u64,
            cdw10: ((IOSQ_SIZE - 1) << 16) | IO_QID,
            cdw11: (IO_QID << 16) | 0x0000_0001,
            ..Default::default()
        };
        self.send_admin_sqe(sqe);
    }

    fn send_admin_sqe(&mut self, sqe: Sqe) {
        Self::send_sqe(
            sqe,
            ASQ_BASE,
            ASQ_SIZE,
            ADMIN_QID,
            self.dstrd,
            &mut self.asq_tail,
        );
        Self::poll_cqe(
            ACQ_BASE,
            ACQ_SIZE,
            ADMIN_QID,
            self.dstrd,
            &mut self.acq_head,
            &mut self.acq_phase,
        );
    }

    fn send_io_sqe(&mut self, sqe: Sqe) {
        Self::send_sqe(
            sqe,
            IOSQ_BASE,
            IOSQ_SIZE,
            IO_QID,
            self.dstrd,
            &mut self.iosq_tail,
        );
        Self::poll_cqe(
            IOCQ_BASE,
            IOCQ_SIZE,
            IO_QID,
            self.dstrd,
            &mut self.iocq_head,
            &mut self.iocq_phase,
        );
    }

    fn send_sqe(sqe: Sqe, sq_base: u32, sq_size: u32, qid: u32, dstrd: u8, sq_tail: &mut u32) {
        let slot = sq_base as usize + *sq_tail as usize * core::mem::size_of::<Sqe>();
        unsafe { core::ptr::write_volatile(slot as *mut Sqe, sqe) }
        cpu::cache::clean(slot, core::mem::size_of::<Sqe>());
        *sq_tail = (*sq_tail + 1) % sq_size;
        unsafe {
            MMIO::<Raw>::base(NVME_DOORBELL_BASE).write([((4 << dstrd) * (2 * qid), *sq_tail)]);
        }
    }

    fn poll_cqe(
        cq_base: u32,
        cq_size: u32,
        qid: u32,
        dstrd: u8,
        cq_head: &mut u32,
        cq_phase: &mut u8,
    ) {
        let slot = cq_base as usize + *cq_head as usize * core::mem::size_of::<Cqe>();
        loop {
            cpu::cache::inval(slot, core::mem::size_of::<Cqe>());
            let cqe = unsafe { core::ptr::read_volatile(slot as *mut Cqe) };
            if cqe.status_phase & 0x1 == *cq_phase as u16 {
                break;
            }
            time::sleep(Duration::from_micros(1));
        }
        *cq_head = (*cq_head + 1) % cq_size;
        if *cq_head == 0 {
            *cq_phase ^= 1;
        }
        unsafe {
            MMIO::<Raw>::base(NVME_DOORBELL_BASE).write([((4 << dstrd) * (2 * qid + 1), *cq_head)]);
        }
    }

    fn alloc_cid(&mut self) -> u16 {
        self.cid = self.cid.wrapping_add(1);
        self.cid
    }
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct Sqe {
    pub cdw0: SqeCw0,
    pub nsid: u32,
    pub rsvd: u64,
    pub mptr: u64,
    pub prp1: u64,
    pub prp2: u64,
    pub cdw10: u32,
    pub cdw11: u32,
    pub cdw12: u32,
    pub cdw13: u32,
    pub cdw14: u32,
    pub cdw15: u32,
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct SqeCw0 {
    pub opcode: u8,
    pub psdt_fuse: u8,
    pub cid: u16,
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct Cqe {
    pub result: u32,
    pub rsvd: u32,
    pub sq_head: u16,
    pub sq_id: u16,
    pub cid: u16,
    pub status_phase: u16,
}

#[allow(unused)]
enum AdminOpcode {
    DeleteIoSq = 0x00,
    CreateIoSq = 0x01,
    GetLogPage = 0x02,
    DeleteIoCq = 0x04,
    CreateIoCq = 0x05,
    Identify = 0x06,
    SetFeatures = 0x09,
}

enum IoOpcode {
    Read = 0x02,
    Write = 0x01,
}

fn init_controller() {
    reset_controller();
    zero_queues();
    configure_admin_queue();
    set_entry_size();
    enable_controller();
}

fn reset_controller() {
    NVME_CTRL.config.modify(Config::ENABLE::CLEAR);
    while !NVME_CTRL.status.matches_all(Status::READY::CLEAR) {
        core::hint::spin_loop();
    }
}

fn zero_queues() {
    const QUEUES_BYTES: usize = (IOCQ_BASE - ASQ_BASE) as usize + 0x1000;
    let queues = unsafe { core::slice::from_raw_parts_mut(ASQ_BASE as *mut u8, QUEUES_BYTES) };
    queues.fill(0);
    cpu::cache::clean(ASQ_BASE as usize, QUEUES_BYTES);
}

fn configure_admin_queue() {
    NVME_CTRL.aqa.write({
        use Aqa::*;
        ASQ_SIZE::ENTRY_64 + ACQ_SIZE::ENTRY_64
    });

    NVME_CTRL.asq_base.write(AsqBase::ADDR::ASQ_BASE);
    NVME_CTRL.acq_base.write(AcqBase::ADDR::ACQ_BASE);
}

fn set_entry_size() {
    NVME_CTRL.config.write({
        use Config::*;
        CSS_NVM::CLEAR
            + MPS_4K::CLEAR
            + AMS_RR::CLEAR
            + SHN_NORMAL::CLEAR
            + IOSQ_ENTRY_SIZE::BYTES_64
            + IOCQ_ENTRY_SIZE::BYTES_16
    });
}

fn enable_controller() {
    NVME_CTRL.config.modify(Config::ENABLE::SET);
    while !NVME_CTRL.status.matches_all(Status::READY::SET) {
        core::hint::spin_loop();
    }
}
