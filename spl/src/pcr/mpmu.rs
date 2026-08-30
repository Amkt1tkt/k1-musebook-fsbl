//! Main PMU (MPMU) at `0xD4050000`.
//!
//! Fixed-frequency clock-output gates. Init raises the whole register; I2C also enables 204.8 MHz.

use tock_registers::{register_bitfields, register_structs, registers::ReadWrite};

use super::MMIO;

/// MPMU MMIO window at `0xD4050000`.
pub const MPMU: MMIO<Mpmu> = unsafe { MMIO::base(0xD405_0000) };

register_structs! {
    /// MPMU register block (clock-gating window used here).
    pub Mpmu {
        (0x0000 => _0x0000),
        (0x1024 => pub clock_gating: ReadWrite<u32, ClockGating::Register>),
        (0x1028 => @END),
    }
}

register_bitfields![u32,
    /// Fixed-frequency clock-output gates.
    pub ClockGating [
        /// Enable the functional 491.52 MHz clock output.
        /// - 1'b0: Clock gated
        /// - 1'b1: Clock enabled
        CLK_491P5M 21,
        /// Enable the functional 12.8 MHz clock output to the Watchdog Timer.
        /// - 1'b0: Clock gated
        /// - 1'b1: Clock enabled
        WDT_12P8M 19,
        /// Enable the functional 245.76 MHz clock output.
        /// - 1'b0: Clock gated
        /// - 1'b1: Clock enabled
        CLK_245P7M 18,
        /// Enable the functional 122.88 MHz clock output.
        /// - 1'b0: Clock gated
        /// - 1'b1: Clock enabled
        CLK_122P8M 17,
        /// Enable the functional 614.4 MHz clock output.
        /// - 1'b0: Clock gated
        /// - 1'b1: Clock enabled
        CLK_61P4M 16,
        /// Enable the functional 819.2 MHz clock output.
        /// - 1'b0: Clock gated
        /// - 1'b1: Clock enabled
        CLK_819P2M 15,
        /// Enable the functional 307.2 MHz clock output.
        /// - 1'b0: Clock gated
        /// - 1'b1: Clock enabled
        CLK_307P2M 14,
        /// Enable the functional 102.4 MHz clock output.
        /// - 1'b0: Clock gated
        /// - 1'b1: Clock enabled
        CLK_102P4M 13,
        /// Enable the functional 51.2 MHz clock output for AP PMU and AP peripherals.
        /// - 1'b0: Clock gated
        /// - 1'b1: Clock enabled
        CLK_51P2_AP 12,
        /// Enable the functional 47.26 MHz clock output.
        /// - 1'b0: Clock gated
        /// - 1'b1: Clock enabled
        CLK_47P2M 11,
        /// Enable the M/N clock generator of the VCXO clock (configured via GPCR); output to `VCXO_OUT` PAD func3.
        /// - 1'b0: Clock gated
        /// - 1'b1: Clock enabled
        GPC 10,
        /// Enable the functional fast UART clock (57.6 MHz) to the Application Processor APB.
        /// - 1'b0: Clock gated
        /// - 1'b1: Clock enabled
        AP_FUART 9,
        /// Enable the functional 51.2 MHz clock output for APB peripherals.
        /// - 1'b0: Clock gated
        /// - 1'b1: Clock enabled
        CLK_51P2M 8,
        /// Enable the functional TWSI clock (31.5 MHz) to the Application Processor APB.
        /// - 1'b0: Clock gated
        /// - 1'b1: Clock enabled
        AP_TWSI 7,
        /// Enable the functional 204.8 MHz clock output.
        /// - 1'b0: Clock gated
        /// - 1'b1: Clock enabled
        CLK_204P8M 6,
        /// Enable the functional 25.6 MHz clock output.
        /// - 1'b0: Clock gated
        /// - 1'b1: Clock enabled
        CLK_25P6M 5,
        /// Enable the functional 12.8 MHz clock output.
        /// - 1'b0: Clock gated
        /// - 1'b1: Clock enabled
        CLK_12P8M 4,
        /// Enable the functional 6.4 MHz clock output.
        /// - 1'b0: Clock gated
        /// - 1'b1: Clock enabled
        CLK_6P4M 3,
        /// Enable the functional slow UART clock (configured via SUCCR, e.g., 14.74 MHz) to the Application Processor APB.
        /// - 1'b0: Clock gated
        /// - 1'b1: Clock enabled
        AP_SUART 1,
        /// Enable the functional 409.6 MHz clock output.
        /// - 1'b0: Clock gated
        /// - 1'b1: Clock enabled
        CLK_409P6M 0,
    ]
];
