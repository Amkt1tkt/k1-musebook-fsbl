//! Branch prediction, prefetch, and L2 snoop enables.
//!
//! Hart 0 sets BPU + prefetch and snoop slot 0. Secondary harts set BPU +
//! prefetch and the L2 snoop bit for `hartid % 4`.

use super::{MSetup, MSetupCSR, Ml2Setup, Ml2SetupCSR};

/// Enable BPU + prefetch, and L2 snoop slot 0 on hart 0.
pub fn enable_perf_features() {
    log::info!("enable BPU / PREFETCH / SNOOP");
    MSetupCSR::enable(MSetup::BPU::SET + MSetup::PREFETCH::SET);
    Ml2SetupCSR::enable(Ml2Setup::SNOOP_0::SET);
}

/// Naked entry: enable BPU + prefetch, then the L2 snoop bit for `hartid % 4`.
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

/// Return `1 << (hartid % 4)` for this hart's cluster-local L2 snoop slot.
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

/// Naked entry: enable BPU + prefetch, then the L2 snoop bit for `hartid % 4`.
#[cfg(not(target_arch = "riscv64"))]
pub unsafe extern "C" fn enable_perf_features_for_secondary_hart() {
    unreachable!()
}
