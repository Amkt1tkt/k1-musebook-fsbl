use super::DDR_TRAIN_VERIFY_BASE;

const TEST_LEN: usize = 64;

pub fn test_pattern() {
    let start = DDR_TRAIN_VERIFY_BASE as *mut u64;
    for offset in 0..TEST_LEN {
        let value = (offset as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0xDEAD_BEEF_CAFE_BABE;
        unsafe { core::ptr::write_volatile(start.add(offset), value) };
    }
    core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
    for offset in 0..TEST_LEN {
        let value = unsafe { core::ptr::read_volatile(start.add(offset)) };
        if value != (offset as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0xDEAD_BEEF_CAFE_BABE {
            panic!("verify test pattern failed at offset {}", offset);
        }
    }
}
