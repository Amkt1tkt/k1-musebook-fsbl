#![no_std]
#![no_main]

use core::arch::naked_asm;

use k1_musebook_flash_server::{nor, rpc, usb};
use k1_musebook_spl::{log, nvme::Nvme, *};

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

#[unsafe(naked)]
unsafe extern "C" fn loop_forever() {
    naked_asm!(
        "wfi",
        "j {loop_forever}",
        loop_forever = sym loop_forever,
    )
}

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

const DDR_STACK_TOP: usize = 0x1000_0000;

// switch stack to DDR because usb receive need more buffer
#[unsafe(naked)]
unsafe extern "C" fn start_listen(nvme: *mut Nvme) -> ! {
    naked_asm!(
        "li sp, {stack_top}",
        "j {rpc_listen}",
        stack_top = const DDR_STACK_TOP,
        rpc_listen = sym rpc_listen,
    )
}

fn rpc_listen(nvme: *mut Nvme) {
    rpc::listen(unsafe { nvme.read() });
}

const BROM_GP: u32 = 0xC083_8C10;

#[unsafe(naked)]
unsafe extern "C" fn restore_brom_gp() {
    naked_asm!(
        "li gp, {brom_gp}",
        "ret",
        brom_gp = const BROM_GP,
    )
}

unsafe extern "C" {
    /// stack top defined in linker script
    /// grows down until bss section
    static STACK_TOP: u8;
}

#[unsafe(naked)]
unsafe extern "C" fn prepare_stack() {
    naked_asm!(
        "la sp, {stack_top}",
        "ret",
        stack_top = sym STACK_TOP,
    )
}

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

#[cfg(target_arch = "riscv64")]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    log::error!("panic: {info:#?}");
    loop {
        core::hint::spin_loop();
    }
}
