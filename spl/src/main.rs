#![no_std]
#![no_main]

use core::{
    arch::naked_asm,
    sync::atomic::{AtomicBool, Ordering},
};

use k1_musebook_spl::{
    gpt::Gpt,
    handoff::FwDynamicInfo,
    layout::{DTB, SBI},
    log,
    nvme::Nvme,
    *,
};

#[unsafe(no_mangle)]
#[unsafe(naked)]
#[cfg_attr(target_arch = "riscv64", unsafe(link_section = ".text.entry"))]
pub unsafe extern "C" fn all_harts_entry() {
    naked_asm!(
        "csrr tp, mhartid",
        "beqz tp, {boot_hart_branch}",
        "bnez tp, {secondary_hart_branch}",
        boot_hart_branch = sym boot_hart_branch,
        secondary_hart_branch = sym secondary_hart_branch,
    )
}

#[unsafe(naked)]
unsafe extern "C" fn boot_hart_branch() {
    naked_asm!(
        "call {prepare_stack}",
        "call {clear_bss}",
        "call {boot}",
        "j {jump_to_sbi}",
        prepare_stack = sym prepare_stack,
        clear_bss = sym clear_bss,
        boot = sym boot,
        jump_to_sbi = sym jump_to_sbi,
    )
}

#[unsafe(naked)]
unsafe extern "C" fn secondary_hart_branch() {
    naked_asm!(
        "call {wait_until_boot_finished}",
        "call {enable_cache}",
        "call {enable_perf_features}",
        "j {jump_to_sbi}",
        wait_until_boot_finished = sym wait_until_boot_finished,
        enable_cache = sym cpu::cache::enable_for_secondary_hart,
        enable_perf_features = sym cpu::enable_perf_features_for_secondary_hart,
        jump_to_sbi = sym jump_to_sbi,
    )
}

pub static FW_DYNAMIC_INFO: FwDynamicInfo = FwDynamicInfo::new();

#[unsafe(naked)]
unsafe extern "C" fn jump_to_sbi() {
    naked_asm!(
        "csrr a0, mhartid",
        "li a1, {dtb_base}",
        "la a2, {fw_dynamic_info}",
        "fence rw, rw",
        "fence.i",
        "li t0, {sbi_base}",
        "jr t0",
        dtb_base = const DTB.load_base,
        fw_dynamic_info = sym FW_DYNAMIC_INFO,
        sbi_base = const SBI.load_base,
    )
}

pub fn boot() {
    log::init();
    log::info!("k1 musebook spl boot start");

    trap::init();
    time::init();

    i2c::init();
    cpu::raise_voltage();
    cpu::raise_freq();

    ddr::init();
    cpu::cache::enable();
    cpu::enable_perf_features();

    pcie::init();
    Gpt::parse(Nvme::open()).load_all_partitions();

    cpu::wake_secondary_harts(all_harts_entry);

    log::info!("k1 musebook spl boot finished");
    IS_BOOT_FINISHED.store(true, Ordering::Release);
}

#[cfg_attr(target_arch = "riscv64", unsafe(link_section = ".data"))]
static IS_BOOT_FINISHED: AtomicBool = AtomicBool::new(false);

#[unsafe(naked)]
unsafe extern "C" fn wait_until_boot_finished() {
    naked_asm!(
        "la t0, {is_boot_finished}",
        "lbu t0, 0(t0)",
        "beqz t0, {wait_until_boot_finished}",
        "ret",
        is_boot_finished = sym IS_BOOT_FINISHED,
        wait_until_boot_finished = sym wait_until_boot_finished,
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
