use tock_registers::interfaces::{ReadWriteable, Readable};

use super::{Control, GENERIC_COUNTER};

pub fn init() {
    log::info!("timer (generic counter) init");
    GENERIC_COUNTER.control.modify(Control::EN::SET);
}

pub fn sleep(duration: core::time::Duration) {
    let start = get_timer_value();
    let sleep_ticks = duration.as_nanos() * get_ticks_per_second() / 1_000_000_000;
    let end = start + (sleep_ticks as u64);
    while get_timer_value() < end {
        core::hint::spin_loop();
    }
}

fn get_timer_value() -> u64 {
    loop {
        let high_1 = GENERIC_COUNTER.value_high.get();
        let low_1 = GENERIC_COUNTER.value_low.get();
        let high_2 = GENERIC_COUNTER.value_high.get();
        if high_1 == high_2 {
            return ((high_1 as u64) << 32) | (low_1 as u64);
        }
    }
}

fn get_ticks_per_second() -> u128 {
    GENERIC_COUNTER.ticks_per_second.get() as u128
}
