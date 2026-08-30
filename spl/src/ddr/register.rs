//! Memory-controller and PHY MMIO windows.
//!
//! Marvell-lineage MC at 0xC000_0000, secure alias at 0xF000_0000, channel at
//! +0x200, PHY control at +0x1000. In-house PHY at 0xC004_0000, with sub-blocks
//! A/B and other-control.

use super::{MMIO, Raw as RawReg};

/// Memory controller at 0xC000_0000.
pub const DDR_CTRL: MMIO<RawReg> = unsafe { MMIO::base(DDR_CTRL_BASE) };
/// Secure alias at 0xF000_0000 used to load frequency-change tables.
pub const DDR_CTRL_SECURE_ALIAS: MMIO<RawReg> = unsafe { MMIO::base(DDR_CTRL_SECURE_ALIAS_BASE) };
/// Per-channel MC registers (base + 0x200).
pub const DDR_CTRL_CHANNEL: MMIO<RawReg> = unsafe { MMIO::base(DDR_CTRL_CHANNEL_BASE) };
/// MC-side PHY control window (base + 0x1000).
pub const DDR_CTRL_PHY_CONTROL: MMIO<RawReg> = unsafe { MMIO::base(DDR_CTRL_PHY_CONTROL_BASE) };

/// In-house PHY at 0xC004_0000.
pub const DDR_PHY: MMIO<RawReg> = unsafe { MMIO::base(DDR_PHY_BASE) };
/// PHY sub-block A.
pub const DDR_PHY_SUB_A: MMIO<RawReg> = unsafe { MMIO::base(DDR_PHY_SUB_A_BASE) };
/// PHY sub-block B.
pub const DDR_PHY_SUB_B: MMIO<RawReg> = unsafe { MMIO::base(DDR_PHY_SUB_B_BASE) };
/// PHY other-control window.
pub const DDR_PHY_OTHER_CONTROL: MMIO<RawReg> = unsafe { MMIO::base(DDR_PHY_OTHER_CONTROL_BASE) };

/// Spacing of per-frequency PHY register windows.
pub const DDR_PHY_FREQ_POINT_STEP: u32 = 0x4000;

/// Memory-controller base address.
pub const DDR_CTRL_BASE: u32 = 0xC000_0000;
/// Secure-alias base used to program the MC internal table.
const DDR_CTRL_SECURE_ALIAS_BASE: u32 = 0xF000_0000;
/// Channel register window offset from the MC base.
pub const DDR_CTRL_CHANNEL_OFFSET: u32 = 0x200;
/// Channel register window base.
pub const DDR_CTRL_CHANNEL_BASE: u32 = DDR_CTRL_BASE + DDR_CTRL_CHANNEL_OFFSET;
/// PHY-control window offset from the MC base.
const DDR_CTRL_PHY_CONTROL_OFFSET: u32 = 0x1000;
/// PHY-control window base.
const DDR_CTRL_PHY_CONTROL_BASE: u32 = DDR_CTRL_BASE + DDR_CTRL_PHY_CONTROL_OFFSET;

/// In-house PHY base address.
const DDR_PHY_BASE: u32 = 0xC004_0000;
/// PHY sub-block A offset from the PHY base.
const DDR_PHY_SUB_A_OFFSET: u32 = 0x3000;
/// PHY sub-block A base.
const DDR_PHY_SUB_A_BASE: u32 = DDR_PHY_BASE + DDR_PHY_SUB_A_OFFSET;
/// PHY sub-block B offset from sub-block A.
const DDR_PHY_SUB_B_OFFSET: u32 = DDR_PHY_SUB_A_OFFSET + 0x200;
/// PHY sub-block B base.
const DDR_PHY_SUB_B_BASE: u32 = DDR_PHY_BASE + DDR_PHY_SUB_B_OFFSET;
/// PHY other-control offset from the PHY base.
const DDR_PHY_OTHER_CONTROL_OFFSET: u32 = 0x1_0000;
/// PHY other-control base.
const DDR_PHY_OTHER_CONTROL_BASE: u32 = DDR_PHY_BASE + DDR_PHY_OTHER_CONTROL_OFFSET;
