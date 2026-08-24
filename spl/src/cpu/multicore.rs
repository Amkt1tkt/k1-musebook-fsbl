use tock_registers::interfaces::{ReadWriteable, Writeable};

use super::{APMU, ClusterXMpIdleCfg, CoreXIdleCfg, CoreXWakeup, cci};

pub fn wake_secondary_harts(entry: unsafe extern "C" fn()) {
    log::info!("wake up secondary harts");
    set_reset_vectors(entry as usize);
    join_cluster_1_into_coherency_domain();
    downgrade_wfi_to_clock_gating();
    wake_up();
}

fn set_reset_vectors(entry: usize) {
    APMU.cluster_0_reset_vector_low.set(entry as u32);
    APMU.cluster_0_reset_vector_high.set((entry >> 32) as u32);
    APMU.cluster_1_reset_vector_low.set(entry as u32);
    APMU.cluster_1_reset_vector_high.set((entry >> 32) as u32);
}

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
