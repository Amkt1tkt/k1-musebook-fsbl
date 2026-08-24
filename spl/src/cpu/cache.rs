use super::{MSetup, MSetupCSR};

pub fn enable() {
    log::info!("enable cpu D/I cache");
    MSetupCSR::enable(MSetup::D_CACHE::SET + MSetup::I_CACHE::SET);
}

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

#[cfg(target_arch = "riscv64")]
pub fn inval(start: usize, len: usize) {
    operate(start, len, |addr| unsafe {
        core::arch::asm!("cbo.inval ({0})", in(reg) addr, options(nostack))
    });
}

#[cfg(target_arch = "riscv64")]
pub fn clean(start: usize, len: usize) {
    operate(start, len, |addr| unsafe {
        core::arch::asm!("cbo.clean ({0})", in(reg) addr, options(nostack))
    });
}

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
#[cfg(not(target_arch = "riscv64"))]
pub unsafe extern "C" fn enable_for_secondary_hart() {
    unreachable!()
}

// Placeholder for host binary compilation
// The actual functionality is only effective on RISC-V firmware
#[cfg(not(target_arch = "riscv64"))]
pub fn inval(_start: usize, _len: usize) {
    unreachable!()
}

// Placeholder for host binary compilation
// The actual functionality is only effective on RISC-V firmware
#[cfg(not(target_arch = "riscv64"))]
pub fn clean(_start: usize, _len: usize) {
    unreachable!()
}
