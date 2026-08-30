//! X60 custom CSR wrappers.
//!
//! MSETUP (0x7C0) controls D/I cache, BPU, and prefetch. ML2SETUP (0x7F0)
//! holds this cluster's L2 snoop bits; each hart enables `hartid % 4`.

use core::marker::PhantomData;

use tock_registers::{RegisterLongName, fields::FieldValue, register_bitfields};

/// Custom CSR at address `ADDR` with bitfield layout `T`.
pub struct CSR<T, const ADDR: u16>(PhantomData<T>);
#[cfg(target_arch = "riscv64")]
impl<T: RegisterLongName, const ADDR: u16> CSR<T, ADDR> {
    /// Custom CSR number used by `csrs` / `csrsi`.
    pub const ADDR: u16 = ADDR;
    /// Set `flags` in this CSR (`csrs`).
    #[inline(always)]
    pub fn enable(flags: FieldValue<usize, T>) {
        unsafe {
            core::arch::asm!(
                "csrs {addr}, {flags}",
                addr = const ADDR,
                flags = in(reg) flags.value,
                options(nomem, nostack),
            )
        }
    }
}

/// MSETUP (0x7C0): D/I cache, BPU, and prefetch.
pub type MSetupCSR = CSR<MSetup::Register, 0x7C0>;
register_bitfields![usize,
    /// D/I cache, BPU, and prefetch enables.
    pub MSetup [
        /// Data-cache enable.
        D_CACHE 0,
        /// Instruction-cache enable.
        I_CACHE 1,
        /// Branch prediction unit.
        BPU 4,
        /// Hardware prefetch.
        PREFETCH 5,
    ]
];

/// ML2SETUP (0x7F0): per-hart L2 snoop bits in this cluster (`hartid % 4`).
pub type Ml2SetupCSR = CSR<Ml2Setup::Register, 0x7F0>;
register_bitfields![usize,
    /// Per-hart L2 snoop bits; each core uses `hartid % 4`.
    pub Ml2Setup [
        /// L2 snoop slot 0 (`hartid % 4 == 0`).
        SNOOP_0 0,
        /// L2 snoop slot 1 (`hartid % 4 == 1`).
        SNOOP_1 1,
        /// L2 snoop slot 2 (`hartid % 4 == 2`).
        SNOOP_2 2,
        /// L2 snoop slot 3 (`hartid % 4 == 3`).
        SNOOP_3 3,
        /// Same bit as `SNOOP_0` (hart 4).
        SNOOP_4 0,
        /// Same bit as `SNOOP_1` (hart 5).
        SNOOP_5 1,
        /// Same bit as `SNOOP_2` (hart 6).
        SNOOP_6 2,
        /// Same bit as `SNOOP_3` (hart 7).
        SNOOP_7 3,
    ]
];

// placeholder for host binary compilation
// The actual functionality is only effective on RISC-V firmware
#[cfg(not(target_arch = "riscv64"))]
impl<T: RegisterLongName, const ADDR: u16> CSR<T, ADDR> {
    /// Set `flags` in this CSR (`csrs`).
    #[inline(always)]
    pub fn enable(_flags: FieldValue<usize, T>) {
        unreachable!()
    }
}
