//! APB system PLL control (APBS) at `0xD4090000`.
//!
//! PLL1/PLL3 SW2 post-divider enables and PLL3 SW3 software force-enable.

use tock_registers::{register_bitfields, register_structs, registers::ReadWrite};

use super::MMIO;

/// APBS MMIO window at `0xD4090000`.
pub const APBS: MMIO<Apbs> = unsafe { MMIO::base(0xD409_0000) };

register_structs! {
    /// APBS register block: PLL1/PLL3 SW2 and PLL3 SW3.
    pub Apbs {
        (0x000 => _0x000),
        (0x104 => pub pll1_sw2_control: ReadWrite<u32, PllXSw2Control::Register>),
        (0x108 => _0x108),
        (0x128 => pub pll3_sw2_control: ReadWrite<u32, PllXSw2Control::Register>),
        (0x12C => pub pll3_sw3_control: ReadWrite<u32, PllXSw3Control::Register>),
        (0x130 => @END),
    }
}

register_bitfields![u32,
    /// PLL1/PLL3 SW2 post-divider enables.
    pub PllXSw2Control [
        /// - internal pin name: {nc,nc,nc, bgsel<2:0>, rtemp<1:0>}
        /// - bg_reg1<7:5>: Reserved
        /// - bg_reg1<4:2>: bandgap output control bits
        /// - bg_reg1<1:0>: bandgap output temperature coefficient control bits
        BG_REG OFFSET(24) NUMBITS(8) [],
        /// Bandgap enable
        /// - 1'b1: Enable
        BG_EN OFFSET(23) NUMBITS(1) [],
        /// REFBUF SW enable
        /// - 1'b1: Enable
        /// - 1'b0: HW control
        REFBUF2_EN OFFSET(22) NUMBITS(1) [],
        /// PLLX divider update enable
        /// - 1'b1: Enable
        /// - 1'b0: Disable
        PLL_UPDATE_EN OFFSET(21) NUMBITS(1) [],
        /// PLLX_DIV23_EN
        /// - 1'b1: Enable
        PLL_DIV23_EN OFFSET(20) NUMBITS(1) [],
        /// PLLX_MON_CFG
        /// [19]: Monitor enable
        /// [18:17]: Monitor divider
        PLL1_MON_CFG OFFSET(17) NUMBITS(3) [],
        /// PLLX_DIV13_EN
        /// - 1'b1: Enable
        PLL_DIV13_EN OFFSET(16) NUMBITS(1) [],
        /// PLLX_DIV11_EN
        /// - 1'b1: Enable
        PLL_DIV11_EN OFFSET(15) NUMBITS(1) [],
        /// DTEST enable
        EN_DTEST OFFSET(14) NUMBITS(1) [],
        /// CKTEST enable
        EN_CKTEST OFFSET(13) NUMBITS(1) [],
        /// ATEST enable
        EN_ATEST OFFSET(12) NUMBITS(1) [],
        /// PLL1_24p576_AUD_EN
        /// - 1'b1: Enable
        PLL_24P576_AUD_EN OFFSET(11) NUMBITS(1) [],
        /// PLL1_245p76_AUD_EN
        /// - 1'b1: Enable
        PLL_245P76_AUD_EN OFFSET(10) NUMBITS(1) [],
        /// PLLX_245p6_DAC_EN
        /// If APBaux/PLL_ADDA_OVRD_EN=1, this bit controls PLLX_245p6_DAC_EN
        /// - 1'b1: Enable
        PLL_245P6_DAC_EN OFFSET(9) NUMBITS(1) [],
        /// PLLX_245p6_ADC_EN
        /// If APBaux/PLL_ADDA_OVRD_EN=1, this bit controls PLLX_245p6_ADC_EN
        /// - 1'b1: Enable
        PLL_245P6_ADC_EN OFFSET(8) NUMBITS(1) [],
        /// PLLX_DIV8_EN
        /// - 1'b1: Enable
        PLL_DIV8_EN OFFSET(7) NUMBITS(1) [],
        /// PLLX_DIV7_EN
        /// - 1'b1: Enable
        PLL_DIV7_EN OFFSET(6) NUMBITS(1) [],
        /// PLLX_DIV6_EN
        /// - 1'b1: Enable
        PLL_DIV6_EN OFFSET(5) NUMBITS(1) [],
        /// PLLX_DIV5_EN
        /// - 1'b1: Enable
        PLL_DIV5_EN OFFSET(4) NUMBITS(1) [],
        /// PLLX_DIV4_EN
        /// - 1'b1: Enable
        PLL_DIV4_EN OFFSET(3) NUMBITS(1) [],
        /// PLLX_DIV3_EN
        /// - 1'b1: Enable
        PLL_DIV3_EN OFFSET(2) NUMBITS(1) [],
        /// PLLX_DIV2_EN
        /// - 1'b1: Enable
        PLL_DIV2_EN OFFSET(1) NUMBITS(1) [],
        /// PLLX_DIV1_EN
        /// - 1'b1: Enable
        PLL_DIV1_EN OFFSET(0) NUMBITS(1) [],
    ],
    /// PLL3 SW3 software force-enable.
    pub PllXSw3Control [
        /// - 1'b0: PLL enable controlled by PMU HW
        /// - 1'b1: SW force enabled
        PLL_SW_EN 31,
    ],
];
