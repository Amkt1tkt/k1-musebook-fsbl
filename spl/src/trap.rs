use riscv::{
    interrupt,
    register::{
        mcause, mepc,
        mie::{self, Mie},
        mtval,
        mtvec::{self, Mtvec, TrapMode},
    },
};

pub fn init() {
    log::info!("trap init");
    disable_all_interrupts();
    set_trap_handler();
}

fn disable_all_interrupts() {
    interrupt::disable();
    unsafe { mie::write(Mie::from_bits(0)) };
}

fn set_trap_handler() {
    unsafe {
        mtvec::write(Mtvec::new(
            trap_handler as *const () as usize,
            TrapMode::Direct,
        ))
    };
}

#[unsafe(no_mangle)]
extern "C" fn trap_handler() -> ! {
    let mcause = mcause::read();
    let mepc = mepc::read();
    let mtval = mtval::read();
    panic!("M-mode trap: mcause=0x{mcause:?} mepc=0x{mepc:x} mtval=0x{mtval:x}");
}
