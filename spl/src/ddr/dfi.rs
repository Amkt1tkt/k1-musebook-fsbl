use super::DDR_CTRL_PHY_CONTROL;

pub fn handshake() {
    unsafe {
        DDR_CTRL_PHY_CONTROL.write([(0x3D0, 0x1300_0001)]);
        DDR_CTRL_PHY_CONTROL.wait_until(0x3FC, |value| value & 0x8000_0000 == 0x8000_0000);
        DDR_CTRL_PHY_CONTROL.write([(0x3D0, 0x1300_0100)]);
    }
    riscv::asm::fence();
}
