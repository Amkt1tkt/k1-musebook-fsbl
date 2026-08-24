#![no_std]

pub mod cci;
pub mod cpu;
pub mod ddr;
pub mod gpt;
pub mod handoff;
pub mod i2c;
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
