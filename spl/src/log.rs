use core::fmt::Write;

pub use ::log::*;

use super::uart;

pub fn init() {
    set_logger(&LOGGER).unwrap();
    set_max_level(LevelFilter::Trace);
}

static LOGGER: Logger = Logger;
pub struct Logger;

impl Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let level = record.level();
        let content = record.args();
        let _ = writeln!(Logger, "[{level}] {content}");
    }

    fn flush(&self) {
        uart::flush();
    }
}

impl core::fmt::Write for Logger {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            if byte == b'\n' {
                uart::write_byte(b'\r');
            }
            uart::write_byte(byte);
        }
        Ok(())
    }
}
