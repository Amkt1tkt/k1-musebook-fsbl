use tock_registers::{register_bitfields, register_structs, registers::ReadWrite};

use super::MMIO;

pub const PCIE_A_PHY_LANE_0: MMIO<PciePhy> = unsafe { MMIO::base(0xC0B1_0000) };
pub const PCIE_A_PHY_LANE_1: MMIO<PciePhy> = unsafe { MMIO::base(0xC0B1_0000 + 0x400) };
pub const PCIE_C_PHY_LANE_0: MMIO<PciePhy> = unsafe { MMIO::base(0xC0D1_0000) };
pub const PCIE_C_PHY_LANE_1: MMIO<PciePhy> = unsafe { MMIO::base(0xC0D1_0000 + 0x400) };

register_structs! {
    pub PciePhy {
        (0x00 => _0x00),
        (0x08 => pub clock_config: ReadWrite<u32, ClockConfig::Register>),
        (0x0C => _0x0c),
        (0x18 => pub pu_rx_config: ReadWrite<u32, PuRxConfig::Register>),
        (0x1C => _0x1c),
        (0x22 => pub rc_cal_reg1: ReadWrite<u8, RcCalReg1::Register>),
        (0x23 => pub rc_cal_reg2: ReadWrite<u8, RcCalReg2::Register>),
        (0x24 => _0x24),
        (0x48 => pub pll_reg1: ReadWrite<u8>),
        (0x49 => pub pll_reg2: ReadWrite<u8, PllReg2::Register>),
        (0x4A => pub pll_reg3: ReadWrite<u8, PllReg3::Register>),
        (0x4B => pub pll_reg4: ReadWrite<u8>),
        (0x4C => pub pll_reg5: ReadWrite<u8, PllReg5::Register>),
        (0x4D => pub pll_reg6: ReadWrite<u8>),
        (0x4E => pub pll_reg7: ReadWrite<u8>),
        (0x4F => pub pll_reg8: ReadWrite<u8>),
        (0x50 => _0x50),
        (0x51 => pub rx_reg1: ReadWrite<u8, RxReg1::Register>),
        (0x52 => pub rx_reg2: ReadWrite<u8>),
        (0x53 => pub rx_reg3: ReadWrite<u8>),
        (0x54 => pub rx_reg4: ReadWrite<u8, RxReg4::Register>),
        (0x55 => _0x55),
        (0x5D => pub refclk_mode: ReadWrite<u8, RefclkMode::Register>),
        (0x5E => _0x5e),
        (0x65 => pub tx_reg1: ReadWrite<u8, TxReg1::Register>),
        (0x66 => pub tx_reg2: ReadWrite<u8>),
        (0x67 => pub tx_reg3: ReadWrite<u8, TxReg3::Register>),
        (0x68 => _0x68),
        (0x84 => pub rterm_calibration_result: ReadWrite<u8, RtermCalibrationResult::Register>),
        (0x85 => pub rterm_calibration_status: ReadWrite<u8, RtermCalibrationStatus::Register>),
        (0x86 => @END),
    }
}

register_bitfields![u32,
    pub ClockConfig [
        FULL OFFSET(0) NUMBITS(32) [
            VALUE_0B78 = 0x0B78,
        ],
        BIT_0 OFFSET(0) NUMBITS(1) [],
    ],
    pub PuRxConfig [
        FORCE_RECIVE_DONE 10,
        PU_RX_LFPS 15,
        MPU_U3 17,
    ],
];

register_bitfields![u8,
    pub RcCalReg1 [
        BIT_6 6,
    ],
    pub RcCalReg2 [
        CAL_REFCLK_FREQ OFFSET(5) NUMBITS(3) [
            MHZ_24 = 0b011,
            MHZ_100 = 0b010,
        ],
    ],
    pub PllReg2 [
        INPUT_FREQ OFFSET(4) NUMBITS(4) [
            REFCLK_MHZ_24 = 0x2,
        ]
    ],
    pub PllReg3 [
        SSC_ENABLE OFFSET(0) NUMBITS(4) []
    ],
    pub PllReg5 [
        OUTPUT_FREQ_MHZ_100 4,
    ],
    pub RxReg1 [
        RTERM_CALIBRATION_LSB OFFSET(0) NUMBITS(4) [],
    ],
    pub RxReg4 [
        BIT_5 5,
    ],
    pub RefclkMode [
        DRIVER 0,
        RECEIVER 1,
        ENABLE 2,
    ],
    pub TxReg1 [
        RTERM_CALIBRATION_MSB OFFSET(4) NUMBITS(4) [],
    ],
    pub TxReg3 [
        BIT_1 1,
    ],
    pub RtermCalibrationResult [
        RTERM_CALIBRATION_LSB OFFSET(0) NUMBITS(4) [],
        RTERM_CALIBRATION_MSB OFFSET(4) NUMBITS(4) [],
    ],
    pub RtermCalibrationStatus [
        DONE 2,
    ],
];
