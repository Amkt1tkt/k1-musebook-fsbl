use super::{
    APBC, APBS, ClockGating, MMIO, MPMU, PINMUX, Pinmux, PllXSw2Control, Twsi8ClockResetControl,
    time,
};

mod operate;
mod register;

pub use self::operate::{init, reset, write};
use self::register::{Control, I2C, Status};
