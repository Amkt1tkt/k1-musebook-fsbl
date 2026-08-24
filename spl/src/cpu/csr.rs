use core::marker::PhantomData;

use tock_registers::{RegisterLongName, fields::FieldValue, register_bitfields};

pub struct CSR<T, const ADDR: u16>(PhantomData<T>);
#[cfg(target_arch = "riscv64")]
impl<T: RegisterLongName, const ADDR: u16> CSR<T, ADDR> {
    pub const ADDR: u16 = ADDR;
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

pub type MSetupCSR = CSR<MSetup::Register, 0x7C0>;
register_bitfields![usize,
    pub MSetup [
        D_CACHE 0,
        I_CACHE 1,
        BPU 4,
        PREFETCH 5,
    ]
];

pub type Ml2SetupCSR = CSR<Ml2Setup::Register, 0x7F0>;
register_bitfields![usize,
    pub Ml2Setup [
        SNOOP_0 0,
        SNOOP_1 1,
        SNOOP_2 2,
        SNOOP_3 3,
        SNOOP_4 0,
        SNOOP_5 1,
        SNOOP_6 2,
        SNOOP_7 3,
    ]
];

// placeholder for host binary compilation
// The actual functionality is only effective on RISC-V firmware
#[cfg(not(target_arch = "riscv64"))]
impl<T: RegisterLongName, const ADDR: u16> CSR<T, ADDR> {
    #[inline(always)]
    pub fn enable(_flags: FieldValue<usize, T>) {
        unreachable!()
    }
}
