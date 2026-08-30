//! Wake secondary harts after joining both clusters to the CCI domain.
//!
//! Writes both clusters' reset vectors, enables cluster 0 snoop, clears
//! cluster 1 MP idle / powerdown / L2 SRAM powerdown, then enables cluster 1
//! snoop. WFI is dropped from C2 deep sleep to clock-gating only (clear
//! `CORE_IDLE` / `PWRDWN` and GIC masks). `CORE0_WAKEUP` then releases harts
//! 1-7.

use tock_registers::interfaces::{ReadWriteable, Writeable};

use super::{APMU, ClusterXMpIdleCfg, CoreXIdleCfg, CoreXWakeup, cci};

/// Program reset vectors, join cluster 1 to CCI, drop WFI from C2, then wake harts 1-7.
pub fn wake_secondary_harts(entry: unsafe extern "C" fn()) {
    log::info!("wake up secondary harts");
    set_reset_vectors(entry as usize);
    join_cluster_1_into_coherency_domain();
    downgrade_wfi_to_clock_gating();
    wake_up();
}

/// Write `entry` into both clusters' reset-vector registers.
fn set_reset_vectors(entry: usize) {
    APMU.cluster_0_reset_vector_low.set(entry as u32);
    APMU.cluster_0_reset_vector_high.set((entry >> 32) as u32);
    APMU.cluster_1_reset_vector_low.set(entry as u32);
    APMU.cluster_1_reset_vector_high.set((entry >> 32) as u32);
}

/// Enable cluster 0 snoop, clear cluster 1 MP idle / powerdown / L2 SRAM powerdown, then enable cluster 1 snoop.
fn join_cluster_1_into_coherency_domain() {
    cci::enable_snoop(&cci::CCI.cluster_0_snoop_control);
    for idle_cfg in [
        &APMU.cluster_1_mp_idle_cfg_core_0,
        &APMU.cluster_1_mp_idle_cfg_core_1,
        &APMU.cluster_1_mp_idle_cfg_core_2,
        &APMU.cluster_1_mp_idle_cfg_core_3,
    ] {
        idle_cfg.modify(
            ClusterXMpIdleCfg::MP_IDLE::CLEAR
                + ClusterXMpIdleCfg::MP_PWRDWN::CLEAR
                + ClusterXMpIdleCfg::L2_SRAM_PWRDWN::CLEAR,
        );
    }
    cci::enable_snoop(&cci::CCI.cluster_1_snoop_control);
}

/// Clear `CORE_IDLE` / `PWRDWN` and GIC masks so WFI only gates clocks (not C2 deep sleep).
fn downgrade_wfi_to_clock_gating() {
    for idle_cfg in [
        &APMU.core_1_idle_cfg,
        &APMU.core_2_idle_cfg,
        &APMU.core_3_idle_cfg,
        &APMU.core_4_idle_cfg,
        &APMU.core_5_idle_cfg,
        &APMU.core_6_idle_cfg,
        &APMU.core_7_idle_cfg,
    ] {
        idle_cfg.modify(
            CoreXIdleCfg::CORE_IDLE::CLEAR
                + CoreXIdleCfg::CORE_PWRDWN::CLEAR
                + CoreXIdleCfg::MASK_GIC_NIRQ_TO_CORE::CLEAR
                + CoreXIdleCfg::MASK_GIC_NFIQ_TO_CORE::CLEAR,
        );
    }
}

/// Set `CORE0_WAKEUP` bits for harts 1-7.
fn wake_up() {
    riscv::asm::fence();
    APMU.core_0_wakeup.write(
        CoreXWakeup::WAKEUP_CORE1::SET
            + CoreXWakeup::WAKEUP_CORE2::SET
            + CoreXWakeup::WAKEUP_CORE3::SET
            + CoreXWakeup::WAKEUP_CORE4::SET
            + CoreXWakeup::WAKEUP_CORE5::SET
            + CoreXWakeup::WAKEUP_CORE6::SET
            + CoreXWakeup::WAKEUP_CORE7::SET,
    );
}
