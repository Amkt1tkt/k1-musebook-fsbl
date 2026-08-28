use core::time::Duration;

use tock_registers::{interfaces::Readable, register_bitfields, registers::InMemoryRegister};

use super::{DDR_CTRL, time};

pub fn init() {
    unsafe {
        DDR_CTRL.write([(0x20, 0x1300_0001)]);
        DDR_CTRL.wait_until(0x8, |value| value & 0x11 == 0x11);
    }
    mr_pulse(0x01);
    mr_pulse(0x02);
    mr_pulse(0x0D);
    mr_pulse(0x03);
    mr_pulse(0x16);
    unsafe {
        DDR_CTRL.write([
            (0x20, 0x1100_2000),
            (0x20, 0x1100_1000),
            (0x20, 0x1200_2000),
            (0x20, 0x1200_1000),
        ]);
    }
    mr_pulse(0x0C);
    mr_pulse(0x0E);
    mr_pulse(0x0B);
    mr_pulse(0x17);
}

#[derive(Copy, Clone)]
pub enum DdrCapacity {
    GB8,
    GB16,
}

pub fn detect_capacity() -> DdrCapacity {
    let capacity = [
        capacity_from_mr8(mr_read(8, 0, 0)),
        capacity_from_mr8(mr_read(8, 1, 0)),
        capacity_from_mr8(mr_read(8, 0, 1)),
        capacity_from_mr8(mr_read(8, 1, 1)),
    ]
    .iter()
    .sum::<u32>();
    log::info!("ddr capacity is {capacity}GB");
    match capacity {
        16 => DdrCapacity::GB16,
        8 => DdrCapacity::GB8,
        _ => panic!("unsupported capacity"),
    }
}

#[derive(Debug)]
pub enum Manufacturer {
    Samsung,
    Hynix,
    Micron,
    Unknown,
}

pub fn detect_manufacturer() -> Manufacturer {
    let mr5 = mr_read(5, 0, 0);
    let manufacturer = match mr5 {
        0x01 => Manufacturer::Samsung,
        0x06 => Manufacturer::Hynix,
        0xFF => Manufacturer::Micron,
        _ => Manufacturer::Unknown,
    };
    log::info!("ddr manufacturer is {manufacturer:?}");
    manufacturer
}

pub fn config_for_capacity(capacity: DdrCapacity) {
    if matches!(capacity, DdrCapacity::GB16) {
        mr_pulse(0x95);
    }
}

register_bitfields![u8,
    pub Mr8 [
        DENSITY OFFSET(2) NUMBITS(4) [
            GB8 = 4,
            GB16 = 6,
        ],
        IO_WIDTH OFFSET(6) NUMBITS(2) [
            X16 = 0b00,
            X8 = 0b01,
        ],
    ]
];

impl Mr8::DENSITY::Value {
    fn to_capacity(self) -> u32 {
        match self {
            Mr8::DENSITY::Value::GB8 => 1,
            Mr8::DENSITY::Value::GB16 => 2,
        }
    }
}

impl Mr8::IO_WIDTH::Value {
    fn to_io_width_multiplier(self) -> u32 {
        match self {
            Mr8::IO_WIDTH::Value::X16 => 1,
            Mr8::IO_WIDTH::Value::X8 => 2,
        }
    }
}

fn capacity_from_mr8(mr8: u8) -> u32 {
    let mr8 = InMemoryRegister::<u8, Mr8::Register>::new(mr8);
    let density = mr8
        .read_as_enum::<Mr8::DENSITY::Value>(Mr8::DENSITY)
        .map(|density| density.to_capacity())
        .unwrap_or(0);
    let io_width_multiplier = mr8
        .read_as_enum::<Mr8::IO_WIDTH::Value>(Mr8::IO_WIDTH)
        .map(|io_width| io_width.to_io_width_multiplier())
        .unwrap_or(0);
    density * io_width_multiplier
}

fn mr_pulse(id: u32) {
    unsafe {
        DDR_CTRL.write([(0x24, 0x1302_0000 | id)]);
    }
    time::sleep(Duration::from_micros(1));
}

fn mr_read(mr: u32, ch: u32, cs: u32) -> u8 {
    unsafe {
        DDR_CTRL.write([(0x24, 0x1001_0000 | ((cs + 1) << 24) | (ch << 18) | mr)]);
        DDR_CTRL.wait_until(0x370, |value| value & (1 << 31) == (1 << 31));
        DDR_CTRL.read(0x370) as u8
    }
}
