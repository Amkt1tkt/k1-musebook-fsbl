use core::time::Duration;

use k1_musebook_spl::{
    cpu, log,
    mmio::{MMIO, Raw},
    time,
};

pub const K1_MUSEBOOK_VID: u16 = 0x361C;
pub const K1_MUSEBOOK_PID: u16 = 0x1001;

/// long(*)(void* buf, u64 len), blocking read USB EP2 OUT
const BROM_USB_RX_FUNC_ADDR: usize = 0xFFE0_37B6;
/// long(*)(const void* buf, u64 len), blocking write USB EP1 IN
const BROM_USB_TX_FUNC_ADDR: usize = 0xFFE0_38D0;

const BROM_USB_CONTROLLER_RUN_FUNC_ADDR: usize = 0xFFE0_3992;
const BROM_USB_POLL_IRQ_FUNC_ADDR: usize = 0xFFE0_35BE;

/// Bulk endpoint max packet size negotiated by the BROM at high speed.
pub const BROM_USB_MAX_PACKET_BYTES: usize = 512;

pub const FRAME_LEN_PREFIX_BYTES: usize = 4;

const RX_BUFFER_BASE: usize = 0x0500_0000;
pub const RX_BUFFER_SIZE: usize = super::protocol::CHUNK_BYTES + 4 * 1024;

const TX_BUFFER_BASE: usize = RX_BUFFER_BASE + RX_BUFFER_SIZE;
pub const TX_BUFFER_SIZE: usize = super::protocol::CHUNK_BYTES + 4 * 1024;

const G_USB_READY_ADDR: usize = 0xC083_84CC;

const USB_CTRL_BASE: usize = 0xC090_0000;
const ENDPTCTRL2_OFFSET: u32 = 0x1C8;
const ENDPTCTRL_RXE_MASK: u32 = 1 << 7;

pub fn init() {
    log::info!("usb init");
    rescue_usb_after_ddr();
    pump_until_reenumerated();
}

pub fn receive(buf: &mut [u8]) -> Result<usize, BromFuncError> {
    let received = unsafe {
        let func: unsafe extern "C" fn(*mut u8, usize) -> isize =
            core::mem::transmute(BROM_USB_RX_FUNC_ADDR);
        func(buf.as_mut_ptr(), buf.len())
    };

    if received < 0 {
        Err(BromFuncError)
    } else {
        cpu::cache::inval(buf.as_ptr() as usize, received as usize);
        handle_detect_stage(buf);
        Ok(received as usize)
    }
}

pub fn send(buf: &[u8]) -> Result<(), BromFuncError> {
    cpu::cache::clean(buf.as_ptr() as usize, buf.len());
    let sent = unsafe {
        let func: unsafe extern "C" fn(*const u8, usize) -> isize =
            core::mem::transmute(BROM_USB_TX_FUNC_ADDR);
        func(buf.as_ptr(), buf.len())
    };

    if sent < 0 { Err(BromFuncError) } else { Ok(()) }
}

pub fn tx_buf() -> &'static mut [u8] {
    unsafe { core::slice::from_raw_parts_mut(TX_BUFFER_BASE as *mut u8, TX_BUFFER_SIZE) }
}

pub fn rx_buf() -> &'static mut [u8] {
    unsafe { core::slice::from_raw_parts_mut(RX_BUFFER_BASE as *mut u8, RX_BUFFER_SIZE) }
}

fn rescue_usb_after_ddr() {
    log::info!("usb rescue after ddr");
    unsafe {
        core::ptr::write_volatile(G_USB_READY_ADDR as *mut u32, 0);
        let func: unsafe extern "C" fn(usize) -> bool =
            core::mem::transmute(BROM_USB_CONTROLLER_RUN_FUNC_ADDR);
        if func(USB_CTRL_BASE) {
            log::error!("controller_run reported FAIL");
        } else {
            log::info!("controller re-inited, waiting re-enum");
        }
    }
}

fn pump_until_reenumerated() {
    log::info!("usb pump until re-enumerated");
    unsafe {
        let usb_poll_irq_once: unsafe extern "C" fn() =
            core::mem::transmute(BROM_USB_POLL_IRQ_FUNC_ADDR);
        let usb_ctrl = MMIO::<Raw>::base(USB_CTRL_BASE as u32);
        usb_ctrl.modify([(ENDPTCTRL2_OFFSET, |x| x & !ENDPTCTRL_RXE_MASK)]);

        while !(usb_ctrl.read(ENDPTCTRL2_OFFSET) & ENDPTCTRL_RXE_MASK != 0) {
            usb_poll_irq_once();
            time::sleep(Duration::from_micros(50));
        }
    }
    log::info!("usb re-enumerated");
}

#[inline(always)]
fn handle_detect_stage(buf: &[u8]) {
    if buf.starts_with(b"getvar:version") {
        log::info!("client detecting stage, will respond with FLASH_SERVER");
        if let Err(err) = send(b"FLASH_SERVER") {
            log::error!("send FLASH_SERVER failed: {err:?}");
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("BROM function returned error")]
pub struct BromFuncError;
