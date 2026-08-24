use super::{MSetup, MSetupCSR, Ml2Setup, Ml2SetupCSR};

pub fn enable_perf_features() {
    log::info!("enable BPU / PREFETCH / SNOOP");
    MSetupCSR::enable(MSetup::BPU::SET + MSetup::PREFETCH::SET);
    Ml2SetupCSR::enable(Ml2Setup::SNOOP_0::SET);
}

#[cfg(target_arch = "riscv64")]
#[unsafe(naked)]
pub unsafe extern "C" fn enable_perf_features_for_secondary_hart() {
    core::arch::naked_asm!(
        "li t0, {m_setup_flags}",
        "csrs {m_setup_addr}, t0",
        "mv s0, ra",
        "call {calculate_snoop_slot}",
        "mv ra, s0",
        "csrs {m_l2_setup_addr}, a0",
        "ret",
        m_setup_addr = const MSetupCSR::ADDR,
        m_setup_flags = const MSetup::BPU::SET.value | MSetup::PREFETCH::SET.value,
        calculate_snoop_slot = sym calculate_snoop_slot_by_hartid_mod_4,
        m_l2_setup_addr = const Ml2SetupCSR::ADDR,
    )
}

/// 4 core per cluster, so hartid % 4 is slot in cluster
#[cfg(target_arch = "riscv64")]
#[unsafe(naked)]
unsafe extern "C" fn calculate_snoop_slot_by_hartid_mod_4() {
    core::arch::naked_asm!(
        "andi t0, tp, 0b11", // t0 = hartid % 4
        "li t1, 1",
        "sll a0, t1, t0", // a0 = 1 << t0
        "ret",
    )
}

#[cfg(not(target_arch = "riscv64"))]
pub unsafe extern "C" fn enable_perf_features_for_secondary_hart() {
    unreachable!()
}
