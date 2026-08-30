//! On-board flash-server entry: hart 0 brings up the board, then listens on DDR.
//!
//! Hart 0 restores BROM `gp` (`0xC0838C10`, required by USB ROM helpers), sets
//! the SRAM stack, clears BSS, raises voltage/frequency, initializes DDR,
//! PCIe/NVMe, NOR, and USB rescue, then moves the stack to DDR `0x10000000`
//! before `listen` (USB RX buffers are too large for SRAM). Secondary harts
//! `wfi` forever.

#![no_std]
#![no_main]

use core::arch::naked_asm;

use k1_musebook_flash_server::{nor, rpc, usb};
use k1_musebook_spl::{log, nvme::Nvme, *};

/// Reset vector: hart 0 enters `main`; other harts `wfi`.
#[unsafe(no_mangle)]
#[unsafe(naked)]
#[cfg_attr(target_arch = "riscv64", unsafe(link_section = ".text.entry"))]
pub unsafe extern "C" fn all_harts_entry() {
    naked_asm!(
        "csrr tp, mhartid",
        "beqz tp, {main_hart_branch}",
        "bnez tp, {secondary_hart_branch}",
        main_hart_branch = sym main_hart_branch,
        secondary_hart_branch = sym loop_forever,
    )
}

/// Hart 0: restore BROM `gp`, set the SRAM stack, clear BSS, run `main`.
#[unsafe(naked)]
unsafe extern "C" fn main_hart_branch() {
    naked_asm!(
        "call {restore_brom_gp}",
        "call {prepare_stack}",
        "call {clear_bss}",
        "call {main}",
        "j {loop_forever}",
        restore_brom_gp = sym restore_brom_gp,
        prepare_stack = sym prepare_stack,
        clear_bss = sym clear_bss,
        main = sym main,
        loop_forever = sym loop_forever,
    )
}

/// Park a hart in `wfi`.
#[unsafe(naked)]
unsafe extern "C" fn loop_forever() {
    naked_asm!(
        "wfi",
        "j {loop_forever}",
        loop_forever = sym loop_forever,
    )
}

/// Bring up clocks, DDR, PCIe/NVMe, NOR, and USB, then listen on the DDR stack.
pub fn main() {
    log::init();
    log::info!("k1 musebook flash server init start");

    time::init();

    i2c::init();
    cpu::raise_voltage();
    cpu::raise_freq();

    ddr::init();
    cpu::enable_perf_features();

    pcie::init();
    let mut nvme = Nvme::open();

    nor::init();
    usb::init();

    log::info!("k1 musebook flash server init finished");

    unsafe { start_listen(&mut nvme as *mut Nvme) };
}

/// DDR stack top used while RPC is listening (`0x10000000`).
const DDR_STACK_TOP: usize = 0x1000_0000;

/// Move `sp` to DDR then jump to `rpc_listen` (USB RX needs a large buffer).
#[unsafe(naked)]
unsafe extern "C" fn start_listen(nvme: *mut Nvme) -> ! {
    naked_asm!(
        "li sp, {stack_top}",
        "j {rpc_listen}",
        stack_top = const DDR_STACK_TOP,
        rpc_listen = sym rpc_listen,
    )
}

/// Enter the postcard-rpc server loop with the opened NVMe handle.
fn rpc_listen(nvme: *mut Nvme) {
    rpc::listen(unsafe { nvme.read() });
}

/// BROM global-pointer value USB ROM functions depend on (`0xC0838C10`).
const BROM_GP: u32 = 0xC083_8C10;

/// Reload BROM `gp` so ROM USB helpers can run.
#[unsafe(naked)]
unsafe extern "C" fn restore_brom_gp() {
    naked_asm!(
        "li gp, {brom_gp}",
        "ret",
        brom_gp = const BROM_GP,
    )
}

unsafe extern "C" {
    /// Linker-script SRAM stack top (grows down toward BSS).
    static STACK_TOP: u8;
}

/// Load `sp` from the linker-script SRAM stack top.
#[unsafe(naked)]
unsafe extern "C" fn prepare_stack() {
    naked_asm!(
        "la sp, {stack_top}",
        "ret",
        stack_top = sym STACK_TOP,
    )
}

/// Zero the BSS range `__bss_start` .. `__bss_end`.
fn clear_bss() {
    unsafe extern "C" {
        static mut __bss_start: u8;
        static mut __bss_end: u8;
    }

    let bss_start = core::ptr::addr_of_mut!(__bss_start);
    let bss_end = core::ptr::addr_of_mut!(__bss_end);
    let bss_size = bss_end as usize - bss_start as usize;

    unsafe {
        core::ptr::write_bytes(bss_start, 0, bss_size);
    }
}

/// Log the panic and spin.
#[cfg(target_arch = "riscv64")]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    log::error!("panic: {info:#?}");
    loop {
        core::hint::spin_loop();
    }
}
