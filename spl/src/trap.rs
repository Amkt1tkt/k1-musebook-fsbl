//! M-mode trap setup for the rest of SPL.
//!
//! Interrupts are masked and `mtvec` is Direct. Any trap prints `mcause`/`mepc`/`mtval` and panics.

use riscv::{
    interrupt,
    register::{
        mcause, mepc,
        mie::{self, Mie},
        mtval,
        mtvec::{self, Mtvec, TrapMode},
    },
};

/// Mask all interrupts and install the direct `mtvec` handler.
pub fn init() {
    log::info!("trap init");
    disable_all_interrupts();
    set_trap_handler();
}

/// Clear `mstatus.MIE` and `mie`.
fn disable_all_interrupts() {
    interrupt::disable();
    unsafe { mie::write(Mie::from_bits(0)) };
}

/// Point `mtvec` at `trap_handler` in Direct mode.
fn set_trap_handler() {
    unsafe {
        mtvec::write(Mtvec::new(
            trap_handler as *const () as usize,
            TrapMode::Direct,
        ))
    };
}

/// Dump `mcause`/`mepc`/`mtval` and panic; never returns.
#[unsafe(no_mangle)]
extern "C" fn trap_handler() -> ! {
    let mcause = mcause::read();
    let mepc = mepc::read();
    let mtval = mtval::read();
    panic!("M-mode trap: mcause=0x{mcause:?} mepc=0x{mepc:x} mtval=0x{mtval:x}");
}
