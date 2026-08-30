//! Volatile MMIO access for SoC register blocks.
//!
//! `MMIO<T>` treats a base address as a typed register struct. `Raw` writes,
//! read-modify-writes, reads, and spins on a `u32` at a byte offset.

use core::{
    marker::PhantomData,
    ops::Deref,
    ptr::{read_volatile, write_volatile},
};

/// Base address viewed as a typed register block `T`.
pub struct MMIO<T>(u32, PhantomData<T>);
impl<T> MMIO<T> {
    /// View `addr` as a `T` register block.
    ///
    /// # Safety
    /// `addr` must be a valid MMIO base for `T` for the rest of the program.
    pub const unsafe fn base(addr: u32) -> Self {
        Self(addr, PhantomData)
    }
}
impl<T> Deref for MMIO<T> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.0 as *const T) }
    }
}

/// Untyped MMIO: access registers by byte offset from the base.
pub struct Raw;
impl MMIO<Raw> {
    /// Write each `(offset, value)` as a little-endian `u32`.
    #[inline(always)]
    pub unsafe fn write<const T: usize>(&self, table: [(u32, u32); T]) {
        for (offset, value) in table {
            let addr = (self.0 + offset) as *mut u32;
            unsafe {
                write_volatile(addr, value);
            }
        }
    }
    /// Read-modify-write each `(offset, func)` as a `u32`.
    #[inline(always)]
    pub unsafe fn modify<const T: usize>(&self, table: [(u32, fn(u32) -> u32); T]) {
        for (offset, func) in table {
            let addr = (self.0 + offset) as *mut u32;
            unsafe {
                let value = read_volatile(addr);
                write_volatile(addr, func(value));
            }
        }
    }
    /// Volatile `u32` read at `offset`.
    #[inline(always)]
    pub unsafe fn read(&self, offset: u32) -> u32 {
        unsafe { read_volatile((self.0 + offset) as *const u32) }
    }
    /// Spin until `condition` is true for the `u32` at `offset`.
    #[inline(always)]
    pub unsafe fn wait_until(&self, offset: u32, condition: fn(u32) -> bool) {
        unsafe {
            while !condition(self.read(offset)) {
                core::hint::spin_loop();
            }
        }
    }
}
