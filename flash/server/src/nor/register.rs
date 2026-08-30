//! FlexSPI/QSPI register block at `0xD420C000` (NXP-style).

use k1_musebook_spl::mmio::MMIO;
use tock_registers::{register_bitfields, register_structs, registers::ReadWrite};

/// NXP-style FlexSPI/QSPI controller at `0xD420C000`.
pub const QSPI: MMIO<Qspi> = unsafe { MMIO::base(0xD420_C000) };

register_structs! {
    pub Qspi {
        (0x000 => pub mcr: ReadWrite<u32, Mcr::Register>),
        (0x004 => _0x004),
        (0x008 => pub ipcr: ReadWrite<u32, Ipcr::Register>),
        (0x00C => _0x00c),
        (0x010 => pub buf0cr: ReadWrite<u32>),
        (0x014 => pub buf1cr: ReadWrite<u32>),
        (0x018 => pub buf2cr: ReadWrite<u32>),
        (0x01C => pub buf3cr: ReadWrite<u32>),
        (0x020 => pub bfgencr: ReadWrite<u32>),
        (0x024 => pub soccr: ReadWrite<u32>),
        (0x028 => _0x028),
        (0x030 => pub buf0ind: ReadWrite<u32>),
        (0x034 => pub buf1ind: ReadWrite<u32>),
        (0x038 => pub buf2ind: ReadWrite<u32>),
        (0x03C => _0x03c),
        (0x100 => pub sfar: ReadWrite<u32>),
        (0x104 => pub sfacr: ReadWrite<u32>),
        (0x108 => pub smpr: ReadWrite<u32>),
        (0x10C => _0x10c),
        (0x110 => pub rbct: ReadWrite<u32>),
        (0x114 => _0x114),
        (0x154 => pub tbdr: ReadWrite<u32>),
        (0x158 => _0x158),
        (0x15C => pub sr: ReadWrite<u32, Sr::Register>),
        (0x160 => pub fr: ReadWrite<u32, Fr::Register>),
        (0x164 => _0x164),
        (0x16C => pub sptrclr: ReadWrite<u32, Sptrclr::Register>),
        (0x170 => _0x170),
        (0x180 => pub sfa1ad: ReadWrite<u32>),
        (0x184 => pub sfa2ad: ReadWrite<u32>),
        (0x188 => pub sfb1ad: ReadWrite<u32>),
        (0x18C => pub sfb2ad: ReadWrite<u32>),
        (0x190 => _0x190),
        (0x200 => pub rbdr: [ReadWrite<u32>; 32]),
        (0x280 => _0x280),
        (0x300 => pub lutkey: ReadWrite<u32, Lutkey::Register>),
        (0x304 => pub lckcr: ReadWrite<u32, Lckcr::Register>),
        (0x308 => _0x308),
        (0x310 => pub lut: [ReadWrite<u32>; 64]),
        (0x410 => @END),
    }
}

register_bitfields![u32,
    pub Mcr [
        ISD OFFSET(16) NUMBITS(4) [
            VALUE_0F = 0x0F,
        ],
        MDIS OFFSET(14) NUMBITS(1) [],
        CLR_TXF OFFSET(11) NUMBITS(1) [],
        CLR_RXF OFFSET(10) NUMBITS(1) [],
        END_CFG OFFSET(2) NUMBITS(2) [
            VALUE_03 = 0x03,
        ],
        SWRSTHD OFFSET(1) NUMBITS(1) [],
        SWRSTSD OFFSET(0) NUMBITS(1) [],
    ],
    pub Ipcr [
        SEQID OFFSET(24) NUMBITS(8) [],
        IDATSZ OFFSET(0) NUMBITS(16) [],
    ],
    pub Sr [
        AHB_ACC 2,
        IP_ACC 1,
        BUSY 0,
    ],
    pub Fr [
        XIP_ON 1,
        TFF 0
    ],
    pub Lutkey [
        FULL OFFSET(0) NUMBITS(32) [
            LUT_UNLOCK = 0x5AF0_5AF0
        ],
    ],
    pub Lckcr [
        LCK_UNLOCK 1,
        LCK_LOCK 0,
    ],
    pub Sptrclr [
        IPPTRC OFFSET(8) NUMBITS(1) [],
        BFPTRC OFFSET(0) NUMBITS(1) [],
    ],
];
