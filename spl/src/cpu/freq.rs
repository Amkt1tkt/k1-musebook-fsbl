//! Raise both CPU clusters to 1600 MHz.
//!
//! Sequence: ungate MPMU clocks, software-force PLL3 on, enable PLL3 /2,
//! then set both clusters' `CX_CLK_SEL` to 1600 MHz and wait for `FC_REQ`
//! to clear.

use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};

use super::{APBS, APMU, ApCpuClusterXClockControl, MPMU, PllXSw2Control, PllXSw3Control};

/// Ungate MPMU clocks, force PLL3 and PLL3/2 on, set both clusters to 1600 MHz, wait for `FC_REQ` to clear.
pub fn raise_freq() {
    log::info!("raise cpu frequency to 1600MHz");
    enable_all_clocks();
    enable_pll3();
    enable_pll3_div2();
    set_cluster_init_freq();
}

/// Set every MPMU clock-gating bit.
fn enable_all_clocks() {
    MPMU.clock_gating.set(u32::MAX);
}

/// Software-force PLL3 on.
fn enable_pll3() {
    APBS.pll3_sw3_control.modify(PllXSw3Control::PLL_SW_EN::SET);
}

/// Enable the PLL3 /2 output.
fn enable_pll3_div2() {
    APBS.pll3_sw2_control
        .modify(PllXSw2Control::PLL_DIV2_EN::SET);
}

/// Set both clusters' `CX_CLK_SEL` to 1600 MHz and spin until `CX_CLK_FC_REQ` clears.
fn set_cluster_init_freq() {
    for cluster in [
        &APMU.ap_cpu_cluster_0_clock_control,
        &APMU.ap_cpu_cluster_1_clock_control,
    ] {
        cluster.modify(
            ApCpuClusterXClockControl::CX_HI_CLK_SEL::CLEAR
                + ApCpuClusterXClockControl::CX_CLK_SEL::MHZ_1600,
        );
        cluster.modify(ApCpuClusterXClockControl::CX_CLK_FC_REQ::SET);
        while !cluster.matches_all(ApCpuClusterXClockControl::CX_CLK_FC_REQ::CLEAR) {
            core::hint::spin_loop();
        }
    }
}
