//! FlexSPI/QSPI NOR operations: AHB Fast Read, IP page program, and 4K sector erase.

use core::time::Duration;

use k1_musebook_spl::{
    pcr::{APMU, apmu::QspiClockResetControl},
    pinmux::{PINMUX, Pinmux},
    time,
};
use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};

use super::{FlashServerError, Fr, Ipcr, Lckcr, Lutkey, Mcr, QSPI, Sptrclr, Sr};

const QSPI_AHB_BASE: u32 = 0xB800_0000;
const QSPI_AHB_SIZE: usize = 0x00D0_0000;
const SEQID_AHB: u32 = 0;
const SEQID_IP: u32 = 1;
const INSTR_STOP: u32 = 0;
const INSTR_CMD: u32 = 1;
const INSTR_ADDR: u32 = 2;
const INSTR_DUMMY: u32 = 3;
const INSTR_READ: u32 = 7;
const INSTR_WRITE: u32 = 8;
const OP_RDSR: u8 = 0x05;
const OP_WREN: u8 = 0x06;
const OP_PP: u8 = 0x02;
const OP_SE: u8 = 0x20;
const PAGE: u32 = 256;
const SECTOR: u32 = 4096;
const TX_POP_MIN: usize = 16;

/// Pinmux, clock, soft-reset, LUT0 Fast Read for AHB, LUT1 for IP commands.
pub fn init() {
    log::info!("nor init");
    config_pinmux();
    reset_clock();
    reset_software();
    enter_disable_mode();
    program_static_registers();
    program_lut_fast_read();
    program_ahb();
    program_memmap();
    enter_normal_mode();
}

/// Copy `dst.len()` bytes from the AHB window at `offset`.
pub fn read(offset: u32, dst: &mut [u8]) -> Result<(), FlashServerError> {
    if dst.is_empty() {
        return Err(FlashServerError::Args);
    }
    let end = (offset as usize)
        .checked_add(dst.len())
        .ok_or(FlashServerError::Args)?;
    if end > QSPI_AHB_SIZE {
        return Err(FlashServerError::Args);
    }
    unsafe {
        core::ptr::copy_nonoverlapping(
            (QSPI_AHB_BASE as usize + offset as usize) as *const u8,
            dst.as_mut_ptr(),
            dst.len(),
        );
    }
    Ok(())
}

/// Program `src` with WREN+PP in 256-byte pages, then invalidate the AHB cache.
pub fn write(offset: u32, src: &[u8]) -> Result<(), FlashServerError> {
    if src.is_empty() {
        return Err(FlashServerError::Args);
    }
    let end = (offset as usize)
        .checked_add(src.len())
        .ok_or(FlashServerError::Args)?;
    if end > QSPI_AHB_SIZE {
        return Err(FlashServerError::Args);
    }
    let mut cur = offset;
    let mut data = src;
    while !data.is_empty() {
        let page_end = (cur / PAGE + 1) * PAGE;
        let chunk = ((page_end - cur) as usize).min(data.len());
        write_enable()?;
        ip_op_write(OP_PP, cur, 3, &data[..chunk])?;
        wait_ready()?;
        cur += chunk as u32;
        data = &data[chunk..];
    }
    ahb_invalidate();
    Ok(())
}

/// 4K SE erase of `[offset, offset+len)`; both must be sector-aligned.
pub fn erase(offset: u32, len: u32) -> Result<(), FlashServerError> {
    if !offset.is_multiple_of(SECTOR) || !len.is_multiple_of(SECTOR) || len == 0 {
        return Err(FlashServerError::Args);
    }
    if (offset as u64 + len as u64) as usize > QSPI_AHB_SIZE {
        return Err(FlashServerError::Args);
    }
    let mut cur = offset;
    let end = offset + len;
    while cur < end {
        write_enable()?;
        ip_op_no_data(OP_SE, cur, 3)?;
        wait_ready()?;
        cur += SECTOR;
    }
    ahb_invalidate();
    Ok(())
}

/// Configure QSPI data/clock/CS pins.
fn config_pinmux() {
    for pin in [
        &PINMUX.qspi_dat_0,
        &PINMUX.qspi_dat_1,
        &PINMUX.qspi_dat_2,
        &PINMUX.qspi_dat_3,
        &PINMUX.qspi_clk,
    ] {
        pin.write({
            use Pinmux::*;
            AF_SEL::FUNCTION_0 + EDGE_CLEAR::SET + DRIVE_2::SET + PULL_SEL::CLEAR
        });
    }
    PINMUX.qspi_cs_1.write({
        use Pinmux::*;
        AF_SEL::FUNCTION_0
            + EDGE_CLEAR::SET
            + DRIVE_2::SET
            + PULL_SEL::SET
            + PULLUP_EN::SET
            + PULLDN_EN::CLEAR
    });
}

/// Enable and divide the QSPI clocks via APMU.
fn reset_clock() {
    APMU.qspi_clock_reset_control.set(0);
    time::sleep(Duration::from_micros(2));

    APMU.qspi_clock_reset_control.write({
        use QspiClockResetControl::*;
        QSPI_CLK_DIV.val(4)
            + QSPI_CLK_SEL::MHZ_106
            + QSPI_CLK_EN::SET
            + QSPI_BUS_CLK_EN::SET
            + QSPI_CLK_RST::SET
            + QSPI_BUS_RST::SET
    });
    time::sleep(Duration::from_micros(10));
}

/// Pulse `SWRSTSD`/`SWRSTHD` once the controller is idle.
fn reset_software() {
    while !(QSPI.sr.matches_all(Sr::BUSY::CLEAR) && QSPI.fr.matches_all(Fr::XIP_ON::CLEAR)) {
        core::hint::spin_loop();
    }

    QSPI.mcr.modify(Mcr::SWRSTSD::SET + Mcr::SWRSTHD::SET);
    time::sleep(Duration::from_micros(1));
    QSPI.mcr.modify(Mcr::SWRSTSD::CLEAR + Mcr::SWRSTHD::CLEAR);
}

/// Set `MDIS` so static registers and LUTs can be programmed.
fn enter_disable_mode() {
    QSPI.mcr.modify(Mcr::MDIS::SET);
}

/// Program SFAR and related static FlexSPI registers.
fn program_static_registers() {
    QSPI.smpr.set(0);
    QSPI.soccr.set(0x8);
    QSPI.sfar.set(QSPI_AHB_BASE);
    QSPI.sfacr.set(0);
}

/// LUT0: `0x0B` Fast Read (CMD + 24-bit addr + 8 dummy + READ).
fn program_lut_fast_read() {
    QSPI.lutkey.write(Lutkey::FULL::LUT_UNLOCK);
    QSPI.lckcr.write(Lckcr::LCK_UNLOCK::SET);
    const BASE: usize = SEQID_AHB as usize * 4;
    QSPI.lut[BASE + 0].set(lut_pair(INSTR_CMD, 0, 0x0B, INSTR_ADDR, 0, 24));
    QSPI.lut[BASE + 1].set(lut_pair(INSTR_DUMMY, 0, 8, INSTR_READ, 0, 0));
    QSPI.lut[BASE + 2].set(lut_pair(INSTR_STOP, 0, 0, INSTR_STOP, 0, 0));
    QSPI.lut[BASE + 3].set(0);
    QSPI.lutkey.write(Lutkey::FULL::LUT_UNLOCK);
    QSPI.lckcr.write(Lckcr::LCK_LOCK::SET);
}

/// Point AHB buffer generation at LUT sequence 0.
fn program_ahb() {
    QSPI.buf0ind.set(0);
    QSPI.buf1ind.set(0);
    QSPI.buf2ind.set(0);
    QSPI.buf0cr.set(0xE);
    QSPI.buf1cr.set(0xE);
    QSPI.buf2cr.set(0xE);
    QSPI.buf3cr.set((1 << 31) | (((512 / 8) & 0xFF) << 8));
    QSPI.bfgencr.set(SEQID_AHB << 12);
}

/// Program AHB memory-map top addresses for the NOR window.
fn program_memmap() {
    const BASE: u32 = QSPI_AHB_BASE;
    const A1: u32 = 10 * 1024 * 1024;
    QSPI.sfa1ad.set((BASE + A1 + 0x00_0000) & 0xFFFF_FC00);
    QSPI.sfa2ad.set((BASE + A1 + 0x10_0000) & 0xFFFF_FC00);
    QSPI.sfb1ad.set((BASE + A1 + 0x20_0000) & 0xFFFF_FC00);
    QSPI.sfb2ad.set((BASE + A1 + 0x30_0000) & 0xFFFF_FC00);
    QSPI.mcr.modify(Mcr::END_CFG::VALUE_03 + Mcr::ISD::VALUE_0F);
}

/// Clear `MDIS` and flush FlexSPI flags.
fn enter_normal_mode() {
    QSPI.mcr.modify(Mcr::MDIS::CLEAR);
    QSPI.rbct.set(1 << 8);
    QSPI.fr.set(0xFFFF_FFFF);
}

/// Data phase programmed into LUT1 for an IP command.
#[derive(Clone, Copy, PartialEq)]
enum DataPhase {
    None,
    Read,
    Write,
}

/// Issue Write Enable (`0x06`).
fn write_enable() -> Result<(), FlashServerError> {
    ip_op_no_data(OP_WREN, 0, 0)
}

/// Poll RDSR until WIP clears, or return [`FlashServerError::Hardware`].
fn wait_ready() -> Result<(), FlashServerError> {
    for _ in 0..500_000 {
        let mut sr = [0u8; 1];
        ip_op_read(OP_RDSR, 0, 0, &mut sr)?;
        if sr[0] & 1 == 0 {
            return Ok(());
        }
        time::sleep(Duration::from_micros(10));
    }
    Err(FlashServerError::Hardware)
}

/// Invalidate the AHB cache with `SWRSTHD`/`SWRSTSD`.
fn ahb_invalidate() {
    QSPI.mcr.modify(Mcr::SWRSTHD::SET + Mcr::SWRSTSD::SET);
    time::sleep(Duration::from_micros(1));
    QSPI.mcr.modify(Mcr::SWRSTHD::CLEAR + Mcr::SWRSTSD::CLEAR);
}

/// Wait until the controller is not busy and has no AHB/IP access.
fn ip_wait_idle() -> Result<(), FlashServerError> {
    for _ in 0..500_000 {
        if QSPI
            .sr
            .matches_all(Sr::BUSY::CLEAR + Sr::IP_ACC::CLEAR + Sr::AHB_ACC::CLEAR)
        {
            return Ok(());
        }
        time::sleep(Duration::from_micros(10));
    }
    Err(FlashServerError::Hardware)
}

/// Run an IP command with no data phase.
fn ip_op_no_data(opcode: u8, addr: u32, addr_bytes: u8) -> Result<(), FlashServerError> {
    ip_wait_idle()?;
    prepare_ip(addr, opcode, addr_bytes, DataPhase::None);
    trigger_ipcr(0);
    Ok(())
}

/// Run an IP command that writes `data` through TBDR.
fn ip_op_write(opcode: u8, addr: u32, addr_bytes: u8, data: &[u8]) -> Result<(), FlashServerError> {
    ip_wait_idle()?;
    prepare_ip(addr, opcode, addr_bytes, DataPhase::Write);
    fill_tx(data);
    trigger_ipcr(data.len() as u32);
    Ok(())
}

/// Run an IP command that reads into `dst` from RBDR.
fn ip_op_read(
    opcode: u8,
    addr: u32,
    addr_bytes: u8,
    dst: &mut [u8],
) -> Result<(), FlashServerError> {
    ip_wait_idle()?;
    prepare_ip(addr, opcode, addr_bytes, DataPhase::Read);
    trigger_ipcr(dst.len() as u32);
    read_rx(dst);
    Ok(())
}

/// Clear FIFOs, set SFAR, and program LUT1 for this IP command.
fn prepare_ip(addr: u32, opcode: u8, addr_bytes: u8, phase: DataPhase) {
    QSPI.mcr.modify(Mcr::CLR_TXF::SET + Mcr::CLR_RXF::SET);
    QSPI.sptrclr
        .modify(Sptrclr::IPPTRC::SET + Sptrclr::BFPTRC::SET);
    QSPI.sfar.set(QSPI_AHB_BASE + addr);
    QSPI.fr.set(QSPI.fr.get());
    program_lut_ip(opcode, addr_bytes, phase);
}

/// Unlock, write LUT1 (CMD / optional ADDR / data / STOP), then lock.
fn program_lut_ip(opcode: u8, addr_bytes: u8, phase: DataPhase) {
    QSPI.lutkey.write(Lutkey::FULL::LUT_UNLOCK);
    QSPI.lckcr.write(Lckcr::LCK_UNLOCK::SET);
    let mut lutval = [0u32; 4];
    let mut idx = 0usize;
    put_lut(&mut lutval, idx, INSTR_CMD, 0, opcode as u32);
    idx += 1;
    if addr_bytes > 0 {
        put_lut(&mut lutval, idx, INSTR_ADDR, 0, addr_bytes as u32 * 8);
        idx += 1;
    }
    match phase {
        DataPhase::None => {}
        DataPhase::Read => {
            put_lut(&mut lutval, idx, INSTR_READ, 0, 0);
            idx += 1;
        }
        DataPhase::Write => {
            put_lut(&mut lutval, idx, INSTR_WRITE, 0, 0);
            idx += 1;
        }
    }
    put_lut(&mut lutval, idx, INSTR_STOP, 0, 0);
    let base = (SEQID_IP * 4) as usize;
    for (i, v) in lutval.iter().enumerate() {
        QSPI.lut[base + i].set(*v);
    }
    QSPI.lutkey.write(Lutkey::FULL::LUT_UNLOCK);
    QSPI.lckcr.write(Lckcr::LCK_LOCK::SET);
}

/// Push `data` into TBDR, padding to the 16-byte TX pop minimum.
fn fill_tx(data: &[u8]) {
    let mut i = 0;
    while i + 4 <= data.len() {
        QSPI.tbdr
            .set(u32::from_le_bytes(data[i..i + 4].try_into().unwrap()));
        i += 4;
    }
    if i < data.len() {
        let mut tmp = [0u8; 4];
        tmp[..data.len() - i].copy_from_slice(&data[i..]);
        QSPI.tbdr.set(u32::from_le_bytes(tmp));
    }
    let mut padded = (data.len() + 3) & !3;
    while padded < TX_POP_MIN {
        QSPI.tbdr.set(0);
        padded += 4;
    }
}

/// Pop `dst` from RBDR.
fn read_rx(dst: &mut [u8]) {
    let mut i = 0;
    while i + 4 <= dst.len() {
        let v = QSPI.rbdr[i / 4].get();
        dst[i..i + 4].copy_from_slice(&v.to_le_bytes());
        i += 4;
    }
    if i < dst.len() {
        let v = QSPI.rbdr[i / 4].get();
        let rest = dst.len() - i;
        dst[i..].copy_from_slice(&v.to_le_bytes()[..rest]);
    }
}

/// Trigger IPCR for LUT1 and wait for TFF plus idle.
fn trigger_ipcr(nbytes: u32) {
    QSPI.ipcr
        .write(Ipcr::SEQID.val(SEQID_IP) + Ipcr::IDATSZ.val(nbytes));

    while !QSPI.fr.matches_all(Fr::TFF::SET) {
        core::hint::spin_loop();
    }
    QSPI.fr.write(Fr::TFF::SET);

    while !QSPI.sr.matches_all(Sr::BUSY::CLEAR) {
        core::hint::spin_loop();
    }
}

/// Pack two LUT instructions into one 32-bit LUT word.
const fn lut_pair(i0: u32, p0: u32, o0: u32, i1: u32, p1: u32, o1: u32) -> u32 {
    let lo = (i0 << 10) | (p0 << 8) | o0;
    let hi = (i1 << 10) | (p1 << 8) | o1;
    lo | (hi << 16)
}

/// Write one LUT instruction at `idx` into a 4-word LUT sequence.
fn put_lut(lut: &mut [u32; 4], idx: usize, instr: u32, pad: u32, opr: u32) {
    let cell = ((instr << 10) | (pad << 8) | (opr & 0xFF)) << ((idx & 1) * 16);
    lut[idx / 2] |= cell;
}
