//! NVMe host driver entry; re-exports `Nvme`.

use super::{
    MMIO, NVME_ACQ_BASE, NVME_ASQ_BASE, NVME_CTRL_BASE, NVME_DMA_SIZE, NVME_IOCQ_BASE,
    NVME_IOSQ_BASE, NVME_READ_DMA_BASE, NVME_READ_DMA_PRP2, Raw, cpu, time,
};

mod operate;
mod register;

pub use self::operate::Nvme;
use self::register::{
    ACQ_SIZE, ADMIN_QID, ASQ_SIZE, AcqBase, Aqa, AsqBase, Cap, Config, IO_QID, IOCQ_SIZE,
    IOSQ_SIZE, NVME_CTRL, NVME_DOORBELL_BASE, Status,
};
