//! K1 dual-cluster CPU bring-up (harts 0-3 / 4-7).
//!
//! Covers voltage, frequency, cache, secondary-hart wake, and BPU / prefetch /
//! snoop.

use super::{
    APBS, APMU, ApCpuClusterXClockControl, ClusterXMpIdleCfg, CoreXIdleCfg, CoreXWakeup, MPMU,
    PllXSw2Control, PllXSw3Control, cci, i2c, time,
};

pub mod cache;
mod csr;
mod freq;
mod multicore;
mod perf;
mod voltage;

use self::csr::{MSetup, MSetupCSR, Ml2Setup, Ml2SetupCSR};
pub use self::{
    cache::enable_for_secondary_hart,
    freq::raise_freq,
    multicore::wake_secondary_harts,
    perf::{enable_perf_features, enable_perf_features_for_secondary_hart},
    voltage::raise_voltage,
};
