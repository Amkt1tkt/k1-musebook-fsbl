//! Power / Clock / Reset (PCR) registers

use super::MMIO;

pub mod apbc;
pub mod apbs;
pub mod apmu;
pub mod mpmu;

pub use self::{
    apbc::{APBC, Twsi8ClockResetControl},
    apbs::{APBS, PllXSw2Control, PllXSw3Control},
    apmu::{
        APMU, ApClockControl, ApCpuClusterXClockControl, ApInterruptMask, ClusterXMpIdleCfg,
        CoreXIdleCfg, CoreXWakeup, DdrCtrlAhb, DdrCtrlHardwareSleepType, DdrPhyLdoControl,
        DdrPhyPll1ControlLow, DdrPhyPll1Enable, DdrPhyPllDiv, PciePortXClockResetControl,
        PciePortXControlLogic,
    },
    mpmu::{ClockGating, MPMU},
};
