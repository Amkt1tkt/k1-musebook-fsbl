use super::FlashServerError;

mod operate;
mod register;

pub use self::operate::{erase, init, read, write};
use self::register::{Fr, Ipcr, Lckcr, Lutkey, Mcr, QSPI, Sptrclr, Sr};
