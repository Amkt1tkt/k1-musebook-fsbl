//! Enable snoop + DVM on a CCI slave interface and wait out `CHANGE_PENDING`.

use tock_registers::{
    interfaces::{Readable, Writeable},
    registers::ReadWrite,
};

use super::{CCI, SnoopControl, Status};

/// Enable snoop + DVM on this interface and spin until `CHANGE_PENDING` clears.
pub fn enable_snoop(snoop_control: &ReadWrite<u32, SnoopControl::Register>) {
    snoop_control.write(SnoopControl::SNOOP_EN::SET + SnoopControl::DVM_EN::SET);
    riscv::asm::fence();
    while CCI.status.is_set(Status::CHANGE_PENDING) {
        core::hint::spin_loop();
    }
}
