//! SpacemiT K1 MUSE Book secondary program loader (SPL).
//!
//! Runs after the BootROM: brings up the CPU, DDR, PCIe, and NVMe.
//! then load later images from GPT partitions into DDR and jump to the SBI.

#![no_std]

pub mod cci;
pub mod cpu;
pub mod ddr;
pub mod gpt;
pub mod handoff;
pub mod i2c;
pub mod layout;
pub mod log;
pub mod mmio;
pub mod nvme;
pub mod pcie;
pub mod pcr;
pub mod pinmux;
pub mod time;
pub mod trap;
pub mod uart;

use self::{
    layout::{
        DDR_TRAIN_VERIFY_BASE, GPT_PARTITIONS, KERNEL, NVME_ACQ_BASE, NVME_ASQ_BASE, NVME_DMA_SIZE,
        NVME_IOCQ_BASE, NVME_IOSQ_BASE, NVME_READ_DMA_BASE, NVME_READ_DMA_PRP2,
    },
    mmio::{MMIO, Raw},
    nvme::Nvme,
    pcie::NVME_CTRL_BASE,
    pcr::{
        APBC, APBS, APMU, ApClockControl, ApCpuClusterXClockControl, ApInterruptMask, ClockGating,
        ClusterXMpIdleCfg, CoreXIdleCfg, CoreXWakeup, DdrCtrlAhb, DdrCtrlHardwareSleepType,
        DdrPhyLdoControl, DdrPhyPll1ControlLow, DdrPhyPll1Enable, DdrPhyPllDiv, MPMU,
        PciePortXClockResetControl, PciePortXControlLogic, PllXSw2Control, PllXSw3Control,
        Twsi8ClockResetControl,
    },
    pinmux::{PINMUX, Pinmux},
};
