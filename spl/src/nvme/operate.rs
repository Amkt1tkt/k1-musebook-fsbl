//! NVMe admin bring-up and 4 KiB chunked I/O.
//!
//! Disables the controller, zeros the DMA queue area, programs 64-entry
//! admin SQ/CQ at 0x4000000, sets CC for 64 B / 16 B entries, enables and
//! waits for RDY, then Create IO CQ/SQ. Reads and writes are sliced into
//! 8-LBA (4 KiB) chunks with PRP at 0x4005000 / 0x4006000. K1 is
//! DMA-noncoherent: clean SQE/write buffers before submit, invalidate
//! CQE/read buffers after completion. Doorbell stride comes from CAP.DSTRD.

use core::time::Duration;

use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};

use super::{
    ACQ_SIZE, ADMIN_QID, ASQ_SIZE, AcqBase, Aqa, AsqBase, Cap, Config, IO_QID, IOCQ_SIZE,
    IOSQ_SIZE, MMIO, NVME_ACQ_BASE, NVME_ASQ_BASE, NVME_CTRL, NVME_DMA_SIZE, NVME_DOORBELL_BASE,
    NVME_IOCQ_BASE, NVME_IOSQ_BASE, NVME_READ_DMA_BASE, NVME_READ_DMA_PRP2, Raw, Status, cpu, time,
};

/// Host-side NVMe controller state (queue pointers and doorbell stride).
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
    /// Logical block size in bytes.
    pub const LBA_BYTES: usize = 512;
    /// I/O chunk size in LBAs (4 KiB).
    const CHUNK_LBAS: usize = 8;
    /// I/O chunk size in bytes.
    const CHUNK_BYTES: usize = Self::CHUNK_LBAS * Self::LBA_BYTES;
    /// Namespace ID used for I/O.
    const NSID: u32 = 1;

    /// Reset the controller, then create admin and I/O queues.
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

    /// Read `dst.len()` bytes starting at `lba` (8-LBA chunks).
    pub fn read(&mut self, lba: u64, dst: &mut [u8]) {
        dst.chunks_mut(Self::CHUNK_BYTES)
            .enumerate()
            .map(|(index, chunk)| (lba + index as u64 * Self::CHUNK_LBAS as u64, chunk))
            .for_each(|(lba, chunk)| {
                self.read_chunk(lba, chunk);
            });
    }

    /// Write `src.len()` bytes starting at `lba` (8-LBA chunks).
    pub fn write(&mut self, lba: u64, src: &[u8]) {
        src.chunks(Self::CHUNK_BYTES)
            .enumerate()
            .map(|(index, chunk)| (lba + index as u64 * Self::CHUNK_LBAS as u64, chunk))
            .for_each(|(lba, chunk)| {
                self.write_chunk(lba, chunk);
            });
    }

    /// Read one chunk via PRP DMA.
    fn read_chunk(&mut self, lba: u64, dst: &mut [u8]) {
        let sqe = Sqe {
            cdw0: SqeCw0 {
                opcode: IoOpcode::Read as u8,
                psdt_fuse: 0,
                cid: self.alloc_cid(),
            },
            nsid: Self::NSID,
            prp1: NVME_READ_DMA_BASE,
            prp2: NVME_READ_DMA_PRP2,
            cdw10: lba as u32,
            cdw11: (lba >> 32) as u32,
            cdw12: (dst.len() / Self::LBA_BYTES - 1) as u32,
            ..Default::default()
        };
        self.send_io_sqe(sqe);
        cpu::cache::inval(NVME_READ_DMA_BASE as usize, dst.len());
        unsafe {
            core::ptr::copy_nonoverlapping(
                NVME_READ_DMA_BASE as usize as *const u8,
                dst.as_mut_ptr(),
                dst.len(),
            );
        }
    }

    /// Write one chunk via PRP DMA.
    fn write_chunk(&mut self, lba: u64, src: &[u8]) {
        unsafe {
            core::ptr::copy_nonoverlapping(
                src.as_ptr(),
                NVME_READ_DMA_BASE as usize as *mut u8,
                src.len(),
            );
        }
        cpu::cache::clean(NVME_READ_DMA_BASE as usize, src.len());
        let sqe = Sqe {
            cdw0: SqeCw0 {
                opcode: IoOpcode::Write as u8,
                psdt_fuse: 0,
                cid: self.alloc_cid(),
            },
            nsid: Self::NSID,
            prp1: NVME_READ_DMA_BASE,
            prp2: NVME_READ_DMA_PRP2,
            cdw10: lba as u32,
            cdw11: (lba >> 32) as u32,
            cdw12: (src.len() / Self::LBA_BYTES - 1) as u32,
            ..Default::default()
        };
        self.send_io_sqe(sqe);
    }

    /// Admin Create I/O Completion Queue.
    fn create_io_cq(&mut self) {
        let sqe = Sqe {
            cdw0: SqeCw0 {
                opcode: AdminOpcode::CreateIoCq as u8,
                psdt_fuse: 0,
                cid: self.alloc_cid(),
            },
            nsid: 0,
            prp1: NVME_IOCQ_BASE as u64,
            cdw10: ((IOCQ_SIZE - 1) << 16) | IO_QID,
            cdw11: 0x0000_0001,
            ..Default::default()
        };
        self.send_admin_sqe(sqe);
    }

    /// Admin Create I/O Submission Queue.
    fn create_io_sq(&mut self) {
        let sqe = Sqe {
            cdw0: SqeCw0 {
                opcode: AdminOpcode::CreateIoSq as u8,
                psdt_fuse: 0,
                cid: self.alloc_cid(),
            },
            nsid: 0,
            prp1: NVME_IOSQ_BASE as u64,
            cdw10: ((IOSQ_SIZE - 1) << 16) | IO_QID,
            cdw11: (IO_QID << 16) | 0x0000_0001,
            ..Default::default()
        };
        self.send_admin_sqe(sqe);
    }

    /// Submit an admin SQE and poll the admin CQE.
    fn send_admin_sqe(&mut self, sqe: Sqe) {
        Self::send_sqe(
            sqe,
            NVME_ASQ_BASE,
            ASQ_SIZE,
            ADMIN_QID,
            self.dstrd,
            &mut self.asq_tail,
        );
        Self::poll_cqe(
            NVME_ACQ_BASE,
            ACQ_SIZE,
            ADMIN_QID,
            self.dstrd,
            &mut self.acq_head,
            &mut self.acq_phase,
        );
    }

    /// Submit an I/O SQE and poll the I/O CQE.
    fn send_io_sqe(&mut self, sqe: Sqe) {
        Self::send_sqe(
            sqe,
            NVME_IOSQ_BASE,
            IOSQ_SIZE,
            IO_QID,
            self.dstrd,
            &mut self.iosq_tail,
        );
        Self::poll_cqe(
            NVME_IOCQ_BASE,
            IOCQ_SIZE,
            IO_QID,
            self.dstrd,
            &mut self.iocq_head,
            &mut self.iocq_phase,
        );
    }

    /// Write an SQE, clean the cache, and ring the SQ doorbell.
    fn send_sqe(sqe: Sqe, sq_base: u64, sq_size: u32, qid: u32, dstrd: u8, sq_tail: &mut u32) {
        let slot = sq_base as usize + *sq_tail as usize * core::mem::size_of::<Sqe>();
        unsafe { core::ptr::write_volatile(slot as *mut Sqe, sqe) }
        cpu::cache::clean(slot, core::mem::size_of::<Sqe>());
        *sq_tail = (*sq_tail + 1) % sq_size;
        unsafe {
            MMIO::<Raw>::base(NVME_DOORBELL_BASE).write([((4 << dstrd) * (2 * qid), *sq_tail)]);
        }
    }

    /// Invalidate and wait for a matching-phase CQE, then ring the CQ doorbell.
    fn poll_cqe(
        cq_base: u64,
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

    /// Allocate the next command identifier.
    fn alloc_cid(&mut self) -> u16 {
        self.cid = self.cid.wrapping_add(1);
        self.cid
    }
}

/// 64-byte NVMe submission queue entry.
#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct Sqe {
    /// Opcode, PSDT/FUSE, and command ID.
    pub cdw0: SqeCw0,
    /// Namespace ID (`0` for admin, `1` for I/O here).
    pub nsid: u32,
    pub rsvd: u64,
    /// Metadata pointer (unused).
    pub mptr: u64,
    /// First PRP / data pointer.
    pub prp1: u64,
    /// Second PRP / PRP list.
    pub prp2: u64,
    pub cdw10: u32,
    pub cdw11: u32,
    pub cdw12: u32,
    pub cdw13: u32,
    pub cdw14: u32,
    pub cdw15: u32,
}

/// CDW0 of an SQE (opcode, flags, CID).
#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct SqeCw0 {
    /// Admin or I/O opcode.
    pub opcode: u8,
    /// PSDT and FUSE flags (always 0 here).
    pub psdt_fuse: u8,
    /// Command identifier.
    pub cid: u16,
}

/// 16-byte NVMe completion queue entry.
#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct Cqe {
    /// Command-specific result.
    pub result: u32,
    pub rsvd: u32,
    /// SQ head pointer after this completion.
    pub sq_head: u16,
    /// SQ identifier.
    pub sq_id: u16,
    /// Matching command ID.
    pub cid: u16,
    /// Status plus phase bit in the LSB.
    pub status_phase: u16,
}

/// Admin command opcodes used here.
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

/// I/O command opcodes used here.
enum IoOpcode {
    Read = 0x02,
    Write = 0x01,
}

/// Disable, zero queues, program admin Q, set entry size, enable.
fn init_controller() {
    reset_controller();
    zero_queues();
    configure_admin_queue();
    set_entry_size();
    enable_controller();
}

/// Clear CC.EN and wait until CSTS.RDY is clear.
fn reset_controller() {
    NVME_CTRL.config.modify(Config::ENABLE::CLEAR);
    while !NVME_CTRL.status.matches_all(Status::READY::CLEAR) {
        core::hint::spin_loop();
    }
}

/// Zero the DMA queue region and clean the cache.
fn zero_queues() {
    let queues = unsafe {
        core::slice::from_raw_parts_mut(NVME_ASQ_BASE as *mut u8, NVME_DMA_SIZE as usize)
    };
    queues.fill(0);
    cpu::cache::clean(NVME_ASQ_BASE as usize, NVME_DMA_SIZE as usize);
}

/// Program AQA / ASQ / ACQ for 64-entry admin queues at 0x4000000.
fn configure_admin_queue() {
    NVME_CTRL.aqa.write({
        use Aqa::*;
        ASQ_SIZE::ENTRY_64 + ACQ_SIZE::ENTRY_64
    });

    NVME_CTRL.asq_base.write(AsqBase::ADDR::ASQ_BASE);
    NVME_CTRL.acq_base.write(AcqBase::ADDR::ACQ_BASE);
}

/// Set CC IOSQES=64 B and IOCQES=16 B.
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

/// Set CC.EN and wait for CSTS.RDY.
fn enable_controller() {
    NVME_CTRL.config.modify(Config::ENABLE::SET);
    while !NVME_CTRL.status.matches_all(Status::READY::SET) {
        core::hint::spin_loop();
    }
}
