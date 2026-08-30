//! Instruction / data cache enable and RISC-V CMO helpers.
//!
//! `enable` sets I/D cache in MSETUP (0x7C0). Secondary harts use a naked
//! `csrsi`. `inval` is `cbo.inval` (DMA→CPU, discard dirty lines); `clean`
//! is `cbo.clean` (CPU→DMA, write-back without drop). Both walk 64-byte
//! lines and finish with `fence` / `fence.i`.

use super::{MSetup, MSetupCSR};

/// Enable I-cache and D-cache in MSETUP (0x7C0).
pub fn enable() {
    log::info!("enable cpu D/I cache");
    MSetupCSR::enable(MSetup::D_CACHE::SET + MSetup::I_CACHE::SET);
}

/// Naked `csrsi` that enables I/D cache in MSETUP on a secondary hart.
#[cfg(target_arch = "riscv64")]
#[unsafe(naked)]
pub unsafe extern "C" fn enable_for_secondary_hart() {
    core::arch::naked_asm!(
        "csrsi {addr}, {flags}",
        "ret",
        addr = const MSetupCSR::ADDR,
        flags = const MSetup::D_CACHE::SET.value | MSetup::I_CACHE::SET.value,
    )
}

/// `cbo.inval` over `[start, start+len)` in 64-byte lines, then fence / fence.i (DMA→CPU; discards dirty lines).
#[cfg(target_arch = "riscv64")]
pub fn inval(start: usize, len: usize) {
    operate(start, len, |addr| unsafe {
        core::arch::asm!("cbo.inval ({0})", in(reg) addr, options(nostack))
    });
}

/// `cbo.clean` over `[start, start+len)` in 64-byte lines, then fence / fence.i (CPU→DMA; write-back, keep).
#[cfg(target_arch = "riscv64")]
pub fn clean(start: usize, len: usize) {
    operate(start, len, |addr| unsafe {
        core::arch::asm!("cbo.clean ({0})", in(reg) addr, options(nostack))
    });
}

/// Apply `operation` to each 64-byte line covering `[start, start+len)`, then `fence` / `fence.i`.
#[cfg(target_arch = "riscv64")]
fn operate(start: usize, len: usize, operation: fn(usize)) {
    (0..len.next_multiple_of(64))
        .step_by(64)
        .map(|offset| start + offset)
        .for_each(operation);
    riscv::asm::fence();
    riscv::asm::fence_i();
}

// Placeholder for host binary compilation
// The actual functionality is only effective on RISC-V firmware
/// Naked `csrsi` that enables I/D cache in MSETUP on a secondary hart.
#[cfg(not(target_arch = "riscv64"))]
pub unsafe extern "C" fn enable_for_secondary_hart() {
    unreachable!()
}

// Placeholder for host binary compilation
// The actual functionality is only effective on RISC-V firmware
/// `cbo.inval` over `[start, start+len)` in 64-byte lines, then fence / fence.i (DMA→CPU; discards dirty lines).
#[cfg(not(target_arch = "riscv64"))]
pub fn inval(_start: usize, _len: usize) {
    unreachable!()
}

// Placeholder for host binary compilation
// The actual functionality is only effective on RISC-V firmware
/// `cbo.clean` over `[start, start+len)` in 64-byte lines, then fence / fence.i (CPU→DMA; write-back, keep).
#[cfg(not(target_arch = "riscv64"))]
pub fn clean(_start: usize, _len: usize) {
    unreachable!()
}
