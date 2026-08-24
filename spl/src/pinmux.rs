use tock_registers::{register_bitfields, register_structs, registers::ReadWrite};

use super::MMIO;

pub const PINMUX: MMIO<PinmuxPins> = unsafe { MMIO::base(0xD401_E000) };

register_structs! {
    pub PinmuxPins {
        (0x000 => _0x000),
        (0x168 => pub qspi_dat_0: ReadWrite<u32, Pinmux::Register>),
        (0x16C => pub qspi_dat_1: ReadWrite<u32, Pinmux::Register>),
        (0x170 => pub qspi_dat_2: ReadWrite<u32, Pinmux::Register>),
        (0x174 => pub qspi_dat_3: ReadWrite<u32, Pinmux::Register>),
        (0x178 => pub qspi_cs_1: ReadWrite<u32, Pinmux::Register>),
        (0x17C => pub qspi_clk: ReadWrite<u32, Pinmux::Register>),
        (0x180 => _0x180),
        (0x228 => pub gpio_118_i2c: ReadWrite<u32, Pinmux::Register>),
        (0x22C => pub gpio_119_i2c: ReadWrite<u32, Pinmux::Register>),
        (0x230 => @END),
    }
}

register_bitfields![u32,
    pub Pinmux [
        /// This field selects between two sets of controls for the pull-up and pull-down functionality as follows:
        /// - 0: The pull-up and pull-down resistors are controlled by the selected alternate function for the pin
        /// - 1: The pull-up and pull-down resistors are controlled by the <PULLUP EN> and <PULLDN EN> fields in this register, overriding the function indicated by the selected alternate function.
        /// During low-power states, this field is overridden to 1 and controlled by the <PULLUP EN> and <PULLDN EN> fields.
        /// In these low-power states, this field is effectively 1, although the register value is not changed (refer to low-power (sleep) mode operation for more information).
        PULL_SEL OFFSET(15) NUMBITS(1) [],
        /// This field controls the output function while the <PULL SEL> field is set to 1 (or is effectively 1) as follows:
        ///   - 0: The internal pull-up resistor of the pin is disabled
        ///   - 1: The internal pull-up resistor of the pin is enabled
        PULLUP_EN OFFSET(14) NUMBITS(1) [],
        /// This field controls the output function while the <PULL SEL> field is set to 1 (or is effectively 1) as follows:
        ///   - 0: The internal pull-down resistor of the pin is disabled
        ///   - 1: The internal pull-down resistor of the pin is enabled
        PULLDN_EN OFFSET(13) NUMBITS(1) [],
        /// This field defines the drive strength and slew rate for this pin (in functional mode when the pin is driving HIGH or LOW value) as follows:
        ///   - 2'b00: SLOW
        ///   - 2'b01: SLOW
        ///   - 2'b10: MEDIUM
        ///   - 2'b11: FAST
        /// They are the DS1 and DS0 bit of the drive strength in the current table.
        DRIVE_1_0 OFFSET(11) NUMBITS(2) [
            SLOW = 1,
            MEDIUM = 2,
            FAST = 3,
        ],
        /// This is the DS2 bit to program for higher level of driving strength in the current table.
        /// The address and reset value is on a pin-by-pin basis. Do not rely on the reset value of this field. It must be configured by software to the desired settings.
        /// For Medium (all GPIOs except for SD card), it is 010.
        /// For Fast (SD card I/O), it is 110.
        DRIVE_2 OFFSET(10) NUMBITS(1) [],
        /// This field controls the Schmitt trigger input threshold as follows:
        ///   - 2'b00: buffer input, threshold is 0.9v
        ///   - 2'b01/10/11: enabled the Schmitt trigger with larger hysteresis for VT- and VT+ threshold (refer to Section [Multi-Function Pin Registers](#36-multi-function-pin-registers) in this chapter)
        ST_1_0 OFFSET(8) NUMBITS(2) [],
        /// This field enables/disables the slew rate output control as follows:
        ///   - 1'b1: Enabled
        ///   - 1'b0: Disabled
        /// Enabling the slew rate output control will slow down the output ramp for EMI considerations.
        SLE OFFSET(7) NUMBITS(1) [],
        /// This field enable/disable the edge-detection logic as follows:
        ///   - 1'b0: Enabled and ready to detect an edge
        ///   - 1'b1: Disabled and no edge is detected
        /// This is an enable for the <EDGE_FALL_EN> and <EDGE_RISE_EN> control fields.
        /// This field is only present when a pin has been defined as potentially waking up on an edge.
        /// If the device is not configured in this manner, this field is not present (i.e. reserved) and writing to it has no effect (refer to Section [Multi-Function I/O Pin Assignments](#34-multi-function-io-pin-assignments) in this chapter for more information about which MFPRs include or not include these bits).
        EDGE_CLEAR OFFSET(6) NUMBITS(1) [],
        /// This field enables/disable to detect a falling edge as follows:
        ///   - 1'b0: Disabled
        ///   - 1'b1: Enable
        /// To detect a falling edge on this pin,
        ///   - The pin needs not be an output
        ///   - This field must be set to 1
        ///   - The <EDGE_CLEAR> field must be set to 0
        /// This field is only present when a pin has been defined as potentially waking up on an edge.
        /// If the device is not configured in this manner, this field is not present (i.e. reserved) and writing to it has no effect (refer to Section [Multi-Function I/O Pin Assignments](#34-multi-function-io-pin-assignments) in this chapter for more information about which MFPRs include or not include these bits).
        EDGE_FALL_EN OFFSET(5) NUMBITS(1) [],
        /// This field enables/disable to detect a rising edge as follows:
        ///   - 1'b0: Disables
        ///   - 1'b1: Enabled
        /// To detect a rising edge on this pin,
        ///   - The pin need not be an output
        ///   - This field must be set to 1
        ///   - The <EDGE_CLEAR> field must be set to 0
        /// This field is only present when a pin has been defined as potentially waking up on an edge.
        /// If the device is not configured in this manner, this field is not present (i.e. reserved) and writing to it has no effect (refer to Section [Multi-Function I/O Pin Assignments](#34-multi-function-io-pin-assignments) in this chapter for more information about which MFPRs include or not include these bits).
        EDGE_RISE_EN OFFSET(4) NUMBITS(1) [],
        /// This field enables/disables a strong pull resistor as follows:
        ///   - 1'b0: Disabled
        ///   - 1'b1: Enabled
        /// This field is used for I2C or SD card PADs which require a strong pull resistor.
        SPU OFFSET(3) NUMBITS(1) [],
        /// This field is used for the selection of an alternate function for a pin between eight possible options as follows:
        ///   - 0x0: Alternate function 0 (always as the primary at reset)
        ///   - 0x1: Alternate function 1
        ///   - 0x2: Alternate function 2
        ///   - 0x3: Alternate function 3
        ///   - 0x4: Alternate function 4
        ///   - 0x5: Alternate function 5
        ///   - 0x6: Alternate function 6
        ///   - 0x7: Alternate function 7
        AF_SEL OFFSET(0) NUMBITS(3) [
            FUNCTION_0 = 0x0,
            FUNCTION_1 = 0x1,
            FUNCTION_2 = 0x2,
            FUNCTION_3 = 0x3,
            FUNCTION_4 = 0x4,
            FUNCTION_5 = 0x5,
            FUNCTION_6 = 0x6,
            FUNCTION_7 = 0x7,
        ],
    ],
];
