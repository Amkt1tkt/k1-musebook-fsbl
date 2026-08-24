use tock_registers::{register_bitfields, register_structs, registers::ReadWrite};

use super::MMIO;

pub const APMU: MMIO<Apmu> = unsafe { MMIO::base(0xD428_2800) };

register_structs! {
    pub Apmu {
        (0x000 => _0x000),
        (0x004 => pub ap_clock_control: ReadWrite<u32, ApClockControl::Register>),
        (0x008 => _0x008),
        (0x060 => pub qspi_clock_reset_control: ReadWrite<u32, QspiClockResetControl::Register>),
        (0x064 => _0x064),
        (0x098 => pub ap_interrupt_mask: ReadWrite<u32, ApInterruptMask::Register>),
        (0x09C => _0x09c),
        (0x0B0 => pub ddr_ctrl_hardware_sleep_type: ReadWrite<u32, DdrCtrlHardwareSleepType::Register>),
        (0x0B4 => _0x0b4),
        (0x0E8 => pub ddr_ctrl_ahb: ReadWrite<u32, DdrCtrlAhb::Register>),
        (0x0EC => _0x0ec),
        (0x124 => pub core_0_idle_cfg: ReadWrite<u32, CoreXIdleCfg::Register>),
        (0x128 => pub core_1_idle_cfg: ReadWrite<u32, CoreXIdleCfg::Register>),
        (0x12C => pub core_0_wakeup: ReadWrite<u32, CoreXWakeup::Register>),
        (0x130 => _0x130),
        (0x160 => pub core_2_idle_cfg: ReadWrite<u32, CoreXIdleCfg::Register>),
        (0x164 => pub core_3_idle_cfg: ReadWrite<u32, CoreXIdleCfg::Register>),
        (0x168 => _0x168),
        (0x304 => pub core_4_idle_cfg: ReadWrite<u32, CoreXIdleCfg::Register>),
        (0x308 => pub core_5_idle_cfg: ReadWrite<u32, CoreXIdleCfg::Register>),
        (0x30C => pub core_6_idle_cfg: ReadWrite<u32, CoreXIdleCfg::Register>),
        (0x310 => pub core_7_idle_cfg: ReadWrite<u32, CoreXIdleCfg::Register>),
        (0x314 => pub cluster_1_mp_idle_cfg_core_0: ReadWrite<u32, ClusterXMpIdleCfg::Register>),
        (0x318 => pub cluster_1_mp_idle_cfg_core_1: ReadWrite<u32, ClusterXMpIdleCfg::Register>),
        (0x31C => pub cluster_1_mp_idle_cfg_core_2: ReadWrite<u32, ClusterXMpIdleCfg::Register>),
        (0x320 => pub cluster_1_mp_idle_cfg_core_3: ReadWrite<u32, ClusterXMpIdleCfg::Register>),
        (0x324 => _0x324),
        (0x38c => pub ap_cpu_cluster_0_clock_control: ReadWrite<u32, ApCpuClusterXClockControl::Register>),
        (0x390 => pub ap_cpu_cluster_1_clock_control: ReadWrite<u32, ApCpuClusterXClockControl::Register>),
        (0x394 => _0x394),
        (0x398 => pub ddr_phy_ldo_control: ReadWrite<u32, DdrPhyLdoControl::Register>),
        (0x39C => pub ddr_phy_pll_1_control_low: ReadWrite<u32, DdrPhyPll1ControlLow::Register>),
        (0x3A0 => _0x3a0),
        (0x3A4 => pub ddr_phy_pll_div: ReadWrite<u32, DdrPhyPllDiv::Register>),
        (0x3A8 => _0x3a8),
        (0x3B4 => pub ddr_phy_pll_1_enable: ReadWrite<u32, DdrPhyPll1Enable::Register>),
        (0x3B8 => _0x3b8),
        (0x3CC => pub pcie_port_a_clock_reset_control: ReadWrite<u32, PciePortXClockResetControl::Register>),
        (0x3D0 => pub pcie_port_a_control_logic: ReadWrite<u32, PciePortXControlLogic::Register>),
        (0x3D4 => pub pcie_port_b_clock_reset_control: ReadWrite<u32, PciePortXClockResetControl::Register>),
        (0x3D8 => pub pcie_port_b_control_logic: ReadWrite<u32, PciePortXControlLogic::Register>),
        (0x3DC => pub pcie_port_c_clock_reset_control: ReadWrite<u32, PciePortXClockResetControl::Register>),
        (0x3E0 => pub pcie_port_c_control_logic: ReadWrite<u32, PciePortXControlLogic::Register>),
        (0x3E4 => _0x3e4),
        (0x5B0 => pub cluster_0_reset_vector_low: ReadWrite<u32>),
        (0x5B4 => pub cluster_0_reset_vector_high: ReadWrite<u32>),
        (0x5B8 => _0x5b8),
        (0x6B0 => pub cluster_1_reset_vector_low: ReadWrite<u32>),
        (0x6B4 => pub cluster_1_reset_vector_high: ReadWrite<u32>),
        (0x6B8 => @END),
    }
}

register_bitfields![u32,
    pub ApClockControl [
        /// Clears `AP_RD_STATUS` by writing logic high to this bit
        AP_RD_ST_CLEAR 31,
        BIT_26 26,
        /// DDR clock frequency change request.
        /// Used to request a change in the DDR clock frequency.
        DDR_FREQ_CHG_REQ 22,
        /// AP speed change voting.
        /// Indicates whether the AP is allowed to change speed.
        AP_ALLOW_SPD_CHG 18,
    ],
    pub QspiClockResetControl [
        /// QSPI_CLK_FC_REQ
        /// - Write 1 to force QSPI_CLK_SEL to work.
        /// - The field is cleared by hardware once the clock switch is done.
        QSPI_CLK_FC_REQ OFFSET(12) NUMBITS(1) [],
        /// QSPI Clock division ratio.
        /// QSPI_Clock_Freq = QSPI_CLK_SEL Freq/(QSPI_CLK_DIV+1)
        QSPI_CLK_DIV OFFSET(9) NUMBITS(3) [],
        /// QSPI_CLK_SEL
        /// - 3'b000: 409 MHz (PLL1_div6)
        /// - 3'b001: 375 MHz (PLL2_div8)
        /// - 3'b010: 307 MHz (PLL1_div8)
        /// - 3'b011: 245 MHz (PLL1_div10)
        /// - 3'b100: 223 MHz (PLL1_div11)
        /// - 3'b101: 106 MHz (PLL1_div23)
        /// - 3'b110: 495 MHz (PLL1_div5)
        /// - 3'b111: 189 MHz (PLL1_div13)
        QSPI_CLK_SEL OFFSET(6) NUMBITS(3) [
            MHZ_409 = 0b000,
            MHZ_375 = 0b001,
            MHZ_307 = 0b010,
            MHZ_245 = 0b011,
            MHZ_223 = 0b100,
            MHZ_106 = 0b101,
            MHZ_495 = 0b110,
            MHZ_189 = 0b111,
        ],
        /// QSPI Function Clock Enable
        /// - 1'b1: Peripheral clock enabled
        /// - 1'b0: Peripheral clock disabled
        QSPI_CLK_EN OFFSET(4) NUMBITS(1) [],
        /// QSPI Bus Clock Enable
        /// - 1'b1: Bus clock enabled
        /// - 1'b0: Bus clock disabled
        QSPI_BUS_CLK_EN OFFSET(3) NUMBITS(1) [],
        /// QSPI clk Reset
        /// - 1'b0: Reset
        QSPI_CLK_RST OFFSET(1) NUMBITS(1) [],
        /// QSPI_BUS_CLK Reset.
        /// - 1'b0: Reset
        QSPI_BUS_RST OFFSET(0) NUMBITS(1) [],
    ],
    pub ApInterruptMask [
        DCLK_FC_DONE_INT_MSK 4,
    ],
    pub DdrCtrlHardwareSleepType [
        /// DDRPHY 0 Enable.
        /// - 1'b1: Enable
        /// - 1'b0: Disable
        DDRP_0_EN OFFSET(30) NUMBITS(1) [],
        /// DCLK Bypass Clock Frequency Change Request.
        /// - Write 1 to trigger a frequency change.
        /// - The field is cleared by hardware once the trigger a frequency change is done.
        DCLK_BYPASS_FC_REQ OFFSET(23) NUMBITS(1) [],
        /// DCLK Bypass Clock Enable.
        /// - 1'b1: Peripheral clock enabled
        /// - 1'b0: Peripheral clock disabled
        DCLK_BYPASS_CLK_EN OFFSET(22) NUMBITS(1) [],
        /// DCLK Bypass Clock Reset.
        /// - 1'b0: Reset
        DCLK_BYPASS_RST OFFSET(21) NUMBITS(1) [],
        /// DCLK Bypass Clock Select.
        /// - 2'b00: PLL1 312 MHz
        /// - 2'b01: PLL1 416 MHz
        /// - 2'b10: 24MHz
        /// - 2'b11: Reserved
        DCLK_BYPASS_SEL OFFSET(19) NUMBITS(2) [
            PLL1_MHZ_312 = 0b00,
            PLL1_MHZ_416 = 0b01,
            MHZ_24 = 0b10,
        ],
        /// DCLK Bypass Clock Divider.
        /// DCLK Bypass Clock = DCLK_BYPASS_SEL / (this field +1).
        /// > Note. Divider only used for Clock source 0 and 1.
        DCLK_BYPASS_DIV OFFSET(16) NUMBITS(3) [],
        /// DCLK Bypass Clock Divider bit 16.
        /// DCLK Bypass Clock = DCLK_BYPASS_SEL / (this field +1).
        /// > Note. Divider only used for Clock source 0 and 1.
        /// Only bit offset 16.
        DCLK_BYPASS_DIV_BIT_16 OFFSET(16) NUMBITS(1) [],
        /// Frequency Change table mask.
        /// - 1'b0: Enabled
        /// - 1'b1: Disabled
        REG_TABLE_EN OFFSET(10) NUMBITS(1) [],
        /// Determines the behavior of the DDRPHY PLL during a frequency change.
        /// - 1'b0: Perform a full PLL switch, including VCO reconfiguration, to transition to a new frequency.
        /// - 1'b1: Keep the VCO frequency unchanged and only update the clock divider or input clock source.
        FREQ_PLL_CHG_MODE OFFSET(9) NUMBITS(1) [],
        /// Memory Controller Register Table Number:
        /// - bit[3]: 1'b1: Frequency change occurs within the same timing table
        /// - bit [2]: 1'b1: The target frequency is a high frequency
        /// - bit[1:0]: Specifies the target timing table number for the memory controller.
        /// Only use bit[1:0] here. So the NUMBITS is 2.
        REG_TABLE_NUM OFFSET(3) NUMBITS(2) [
            MT_1200 = 0b00,
            MT_1600 = 0b01,
            MT_2400 = 0b10,
            MT_3200 = 0b11,
        ],
    ],
    pub DdrCtrlAhb [
        /// Disable Dynamic Frequency Change during D1P.
        /// - 1'b1: Disable
        /// - 1'b0: Enable
        DFC_D1P_BLOCK 31,
        /// DDR DPHY PU control.
        /// - 1'b1: Enable
        /// - 1'b0: Disable
        DDR_DPHY_PU 30,
        /// Memory Controller Clock Gating Bypass.
        /// - 1'b1: Bypass MCK_root clock gating during low power state.
        /// - 1'b0: No Bypass, MCK_root is gated during low power state (for power saving)
        CLK_GATE_BYPS 29,
        /// Memory Controller AHB Clock Enable.
        /// - 1'b1: AHB clock enabled
        /// - 1'b0: AHB clock disabled
        AHBCLK_EN 1,
        /// Memory Controller HCLK Reset.
        /// - 1'b0: Reset
        HCLK_RST 0,
    ],
    pub CoreXIdleCfg [
        /// Mask core clock off check during core idle process
        MASK_CLK_OFF_CHECK 11,
        /// Mask core clock stable check during core wakeup
        MASK_CLK_STBL_CHECK 10,
        /// Mask the JTAG idle check during MP idle entry
        MASK_JTAG_IDLE_CHECK 9,
        /// Mask the Core WFI IDLE check during MP idle entry
        MASK_CORE_WFI_IDLE_CHECK 8,
        /// Mask nFIQ generated in GIC for CORE.
        /// Software can set this bit before CORE enters C2.
        /// APMU hardware will automatically clear this bit when CORE enters C2.
        MASK_GIC_NFIQ_TO_CORE 4,
        /// Mask nIRQ generated in GIC for CORE.
        /// Software can set this bit before CORE enters C2.
        /// APMU hardware will automatically clear this bit when CORE enters C2.
        MASK_GIC_NIRQ_TO_CORE 3,
        /// Core Power Down.
        /// This bit does not take effect if CORE_IDLE is 0.
        /// - 1'b1: When core issues WFI idle, core goes into deep sleep mode and power is off
        CORE_PWRDWN 1,
        /// Core Idle.
        /// - 1'b1: When core issues WFI idle, the core clock will be gated externally
        CORE_IDLE 0,
    ],
    pub CoreXWakeup [
        WAKEUP_CORE7 7,
        WAKEUP_CORE6 6,
        WAKEUP_CORE5 5,
        WAKEUP_CORE4 4,
        WAKEUP_CORE3 3,
        WAKEUP_CORE2 2,
        WAKEUP_CORE1 1,
        WAKEUP_CORE0 0,
    ],
    pub ClusterXMpIdleCfg [
        /// Disable the MP L2 power switch sleep power down during MP power down mode
        DIS_MP_L2_SLP 19,
        /// Disable the MP power switch sleep power down during MP subsystem power down mode
        DIS_MP_SLP 18,
        /// Frequency Change L2 SRAM Off.
        /// 1'b1: L2 Frequency Change is off
        FRC_L2_SRAM_OFF 16,
        /// L2 Hardware Cache Flush Enable
        L2_HW_CACHE_FLUSH_EN 13,
        /// Mask SRAM Repair Done Check
        MASK_SRAM_REPAIR_DONE_CHECK 12,
        /// Mask the MP clock off check during the MP idle process
        MASK_CLK_OFF_CHECK 11,
        /// Mask the MP clock stable check during MP wakeup
        MASK_CLK_STBL_CHECK 10,
        /// Mask the JTAG idle check during the MP idle entry
        MASK_JTAG_IDLE_CHECK 9,
        /// Mask the MP Idle State Check
        MASK_IDLE_CHECK 8,
        /// ACINACTM Hardware Control.
        /// 1'b1: low power state machine controls ACINACTM port of MP;
        /// when M2/M1 low power mode is entered, ACINACTM port will be high
        ACINACTM_HW_CTRL 7,
        /// Disable the Memory Controller entry to idle mode using the sleep request bits
        DIS_MC_SW_REQ 5,
        /// Wake up the Memory Controller when the MP wakes up from idle mode
        MP_WAKE_MC_EN 4,
        /// L2 Cache SRAM Power Down.
        /// This field does not take effect if MP_PWRDWN is 0.
        /// - 1'b1: When MP is idle, L2 SRAM power will be off
        /// - 1'b0: When MP is idle, L2 SRAM is in retention mode
        L2_SRAM_PWRDWN 2,
        /// MP Power Down.
        /// This field does not take effect if MP_IDLE is 0.
        /// - 1'b1: When MP is idle, MP will go into deep sleep mode and the MP logic will be power-gated
        MP_PWRDWN 1,
        /// MP Idle.
        /// - 1'b1: When MP is idle, the MP clocks will be gated externally
        MP_IDLE 0,
    ],
    pub ApCpuClusterXClockControl [
        /// CPU clusterX highest Clock Frequecny Selection.
        /// It controls the selection of the highest clock frequency from CPU Cluster X based on the configuration of PLL3
        /// - 1'b0: PLL3_div2(1600MHz) if PLL3 VCO is 3200M
        /// - 1'b1: PLL3_div1(1600MHz) if PLL3 VCO is 1600M
        CX_HI_CLK_SEL OFFSET(13) NUMBITS(1) [],
        /// CPU clusterX clk frequency change request.
        /// - 1'b1: Enable clock frequency change.
        /// - The field is cleared by hardware once the frequency change is done.
        CX_CLK_FC_REQ OFFSET(12) NUMBITS(1) [],
        /// Clock Divider Selection for ClusterX TCM AXI slave Clock.
        /// Formula:CX_TCM_AXI = CX_CORE_CLK / (this field +1)
        CX_TCM_AXI_DIV OFFSET(9) NUMBITS(3) [],
        /// Clock Divider Selection for ClusterX ACE Interface Clock.
        /// Formula:CX_ACE_CLK = CX_CORE_CLK / (this field +1)
        CX_ACE_CLK_DIV OFFSET(6) NUMBITS(3) [],
        /// Clock Divider Selection for CX_CORE_CLK.
        /// Formula:CX_CORE_CLK = Clock Selection / (this field +1)
        CX_CORE_CLK_DIV OFFSET(3) NUMBITS(3) [],
        /// CPU ClusterX Clock Selection
        /// - 3'b000: 614MHz
        /// - 3'b001: 819MHz
        /// - 3'b010: 409MHz
        /// - 3'b011: 491MHz
        /// - 3'b100: 1228MHz
        /// - 3'b101: PLL3_div3(1066MHz)
        /// - 3'b110: PLL2_div3_gated(1000MHz)
        /// - 3'b111: depends on bit[13]:
        ///     1'b0: PLL3_div2(1600MHz)
        ///     1'b1: PLL3_div1(1600MHz)
        CX_CLK_SEL OFFSET(0) NUMBITS(3) [
            MHZ_614 = 0b000,
            MHZ_1600 = 0b111,
        ],
    ],
    pub DdrPhyLdoControl [
        BIT_10_11 OFFSET(10) NUMBITS(2) [],
    ],
    pub DdrPhyPll1ControlLow [
        BYTE_1 OFFSET(8) NUMBITS(8) [
            MT_1200 = 0x3B,
        ],
    ],
    pub DdrPhyPllDiv [
        BYTE_1 OFFSET(8) NUMBITS(8) [
            VALUE_0F = 0x0F,
        ],
    ],
    pub DdrPhyPll1Enable [
        BIT_16_17 OFFSET(16) NUMBITS(2) [],
        BIT_11 OFFSET(11) NUMBITS(1) [],
        BIT_8 OFFSET(8) NUMBITS(1) [],
        BIT_9 OFFSET(9) NUMBITS(1) [],
        FREQ OFFSET(0) NUMBITS(32) [
            MT_1200 = 0x00003B50,
            MT_1600 = 0x00003B04,
            MT_2400 = 0x00003B40,
            MT_3200 = 0x00003B00,
            EXTERNAL_CLOCK = 0x00003B02,
        ],
    ],
    pub PciePortXClockResetControl [
        /// PCIe mode selection:
        /// - 1'b0: EP
        /// - 1'b1: RC
        PCIE_DEVICE_TYPE_SEL OFFSET(31) NUMBITS(1) [],
        /// Set this signal to 1 before the de-assertion of power-on reset sequence to hold the PHY in reset.
        /// This can be used for PHY configuration.
        PCIE_APP_HOLD_PHY_RST OFFSET(30) NUMBITS(1) [],
        /// Used to enable or disable SRIS mode for PCIe controller
        PCIE_APP_SRIS_MODE OFFSET(29) NUMBITS(1) [],
        /// Device number for RC mode
        PCIE_APP_DEV_NUM OFFSET(24) NUMBITS(8) [],
        /// Bus number for RC mode
        PCIE_APP_BUS_NUM OFFSET(16) NUMBITS(8) [],
        /// Wake Up.
        /// Used to wake the PCIe controller from low-power states (L1/L2) and restore active operation.
        /// When the PME is enabled and configured in the PMCSR, asserting this signal wakes the controller from L1 or L2 states;
        /// once the controller transitions back to the L0 state, it sends a PME message and sets the PME_Status.
        /// The root complex then clears PME_Status and changes the D-state back to D0.
        PCIE_APPS_PM_XMT_PME OFFSET(15) NUMBITS(1) [],
        /// DBI Read-only Write Disabled
        /// Controls the write access behavior of the DBI_RO_WR_EN register field.
        /// - 1'b0: MISC_CONTROL_1_OFF
        /// 1. DBI_RO_WR_EN register field is read-write.
        /// - 1'b1: MISC_CONTROL_1_OFF
        /// 1. DBI_RO_WR_EN register field is forced to 0 and is read-only.
        PCIE_APP_DBI_RO_WR_DISABLE OFFSET(14) NUMBITS(1) [],
        /// In EP mode, SE can program this bit to 1 to drive WAKE# to low.
        /// This is a wakeup event for RC side
        PCIE_EP_WAKE_SW OFFSET(13) NUMBITS(1) [],
        /// In RC mode, SW can program this bit to 1 to drive PERST# to low.
        /// This is a WARM reset for EP side
        PCIE_RC_PERST OFFSET(12) NUMBITS(1) [],
        /// If this bit is set to 1, the chip drives the CLKREQ# signal for PortX to 0
        PCIE_PORTX_CLKREQ_OE OFFSET(11) NUMBITS(1) [],
        /// Show the value of PortX CLKREQ# IO input value
        PCIE_PORTX_CLKREQ_IN OFFSET(10) NUMBITS(1) [],
        /// Auxiliary Power Detected
        /// Used to report to the host software that auxiliary power (Vaux) is present
        PCIE_SYS_AUX_PWR_DET OFFSET(9) NUMBITS(1) [],
        /// Global reset
        /// Software (SW) must clear this bit to 0 while simultaneously asserting the following reset signals:
        /// - pcie_axi_dbi_resetn
        /// - pcie_axi_slv_resetn
        /// - pcie_axi_mstr_resetn
        /// Note: This reset signal is high-level-valid
        PCIE_GLB_RST OFFSET(8) NUMBITS(1) [],
        /// PERST value form PAD for EP mode
        PCIE_PERSTN_IN OFFSET(7) NUMBITS(1) [],
        /// Enable the PCIe controller to start training
        /// - 1'b1: Enable
        /// - 1'b0: Hold the ltssm in detect.
        PCIE_LTSSM_EN OFFSET(6) NUMBITS(1) [],
        /// PCIe AXI data master port reset-n.
        /// - 1'b1: Non-Reset
        /// - 1'b0: Reset
        PCIE_AXI_MSTR_RESETN OFFSET(5) NUMBITS(1) [],
        /// PCIe AXI data slave port reset-n
        /// - 1'b1: Non-Reset
        /// - 1'b0: Reset
        PCIE_AXI_SLV_RESETN OFFSET(4) NUMBITS(1) [],
        /// PCIe AXI DBI slave port resetn
        /// - 1'b1: Non-Reset
        /// - 1'b0: Reset
        PCIE_AXI_DBI_RESETN OFFSET(3) NUMBITS(1) [],
        /// PCIe AXI data master port clock enable
        /// - 1'b1: Enable
        /// - 1'b0: Disable
        PCIE_AXI_MSTR_CLK_EN OFFSET(2) NUMBITS(1) [],
        /// PCIe AXI data slave port clock enable
        /// - 1'b1: Enable
        /// - 1'b0: Disable
        PCIE_AXI_SLV_CLK_EN OFFSET(1) NUMBITS(1) [],
        /// PCIe AXI DBI slave port clock enable
        /// - 1'b1: Enable
        /// - 1'b0: Disable
        PCIE_AXI_DBI_CLK_EN OFFSET(0) NUMBITS(1) [],
    ],
    pub PciePortXControlLogic [
        /// Used to configure the debounce settings for the PCIe Root Complex (RC) WAKE_N signal
        PCIE_RC_WAKEN_DEB_CFG OFFSET(20) NUMBITS(2) [],
        /// Used to configure the debounce settings for the PCIe PERST# input signal
        PCIE_PERSTN_IN_DEB_CFG OFFSET(18) NUMBITS(2) [],
        /// Used to configure the debounce settings for the PCIe RX Electrical Idle signal
        PCIE_RXELECIDLE_DEB_CFG OFFSET(16) NUMBITS(2) [],
        /// Used to enable the PCIe wake-up interrupt
        PCIE_WAKEUP_INT_EN OFFSET(15) NUMBITS(1) [],
        /// Used to enable the wake-up functionality of the PCIe device itself
        PCIE_WAKEUP_EN OFFSET(14) NUMBITS(1) [],
        /// Used to indicate the PCIe wakeup interrupt status:
        /// - bit 13: PCIe RC wakeup event
        /// - bit 12: PCIe EP perstn wakeup event
        /// - bit 11: PCIe RX Electrical Idle wakeup event
        PCIE_WAKEUP_INT_REG OFFSET(11) NUMBITS(3) [],
        /// Used to clear the wake-up interrupt status bits for various PCIe wake-up events
        /// - bit 10: PCIe RC wakeup event
        /// - bit 9: PCIe EP perstn wakeup event
        /// - bit 8: PCIe RX Electrical Idle wakeup event
        PCIE_WAKEUP_INT_CLR OFFSET(8) NUMBITS(3) [],
        /// PCIe wake-up interrupt mask
        /// - bit 6: PCIe RC wakeup event
        /// - bit 5: PCIe EP perstn wakeup event
        /// - bit 4: PCIe RX Electrical Idle wakeup event
        PCIE_WAKEUP_MASK OFFSET(4) NUMBITS(3) [],
        /// Wake# Source Selection in EP mode
        /// - 1'b1: The WAKE# pad is driven by pcie_ep_wake_sw bit of PCIe CLK Reset Control Register
        /// - 1'b0: The WAKE# pad is driven by PCIe controller
        PCIE_WAKE_SOURCE_SEL OFFSET(3) NUMBITS(1) [],
        /// Used to control whether the PCIe controller and PHY in EP mode respond to the PERSTN signal from the RC.
        /// - When this bit is set to 1, The PCIe controller and PHY ignore the PERSTN signal from the RC
        PCIE_IGNORE_PERSTN OFFSET(2) NUMBITS(1) [],
        /// In EP mode, SW can set this bit to 1 to force the PERST# signal to be asserted
        PCIE_FORCE_PERSTN OFFSET(1) NUMBITS(1) [],
        /// PCIe soft reset
        /// - 1'b1: Reset
        /// - 1'b0: Non-Reset
        PCIE_SOFT_RESET OFFSET(0) NUMBITS(1) [],
    ],
];
