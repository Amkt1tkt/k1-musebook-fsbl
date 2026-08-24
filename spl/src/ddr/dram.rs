use super::DDR_CTRL;

pub fn init() {
    unsafe {
        DDR_CTRL.write([(0x20, 0x1300_0001)]);
        DDR_CTRL.wait_until(0x8, |value| value & 0x11 == 0x11);
    }
}
