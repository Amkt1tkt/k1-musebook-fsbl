use super::{DDR_CTRL_BASE, DdrFreq, cpu, image::TRAIN_IMAGE};

const TRAIN_IMAGE_START_ADDR: usize = 0xC083_2000;

pub fn train(freq: DdrFreq) {
    unsafe {
        core::ptr::copy_nonoverlapping(
            TRAIN_IMAGE.as_ptr(),
            TRAIN_IMAGE_START_ADDR as *mut u8,
            TRAIN_IMAGE.len(),
        );
    }

    cpu::cache::clean(TRAIN_IMAGE_START_ADDR, TRAIN_IMAGE.len());

    unsafe extern "C" fn training_printf_stub(_a0: usize) -> i32 {
        0
    }
    static mut TRAINING_INPUT_BUF: [u8; 4096] = [0; 4096];
    let printf_ptr = training_printf_stub as unsafe extern "C" fn(usize) -> i32 as usize;
    let input_ptr = (&raw mut TRAINING_INPUT_BUF) as *mut u8 as usize;

    unsafe {
        training_entry(TRAIN_IMAGE_START_ADDR)(&TrainingParams {
            ddr_ctrl_base: DDR_CTRL_BASE as u64,
            cs_num: 2_u64,
            freq: freq as u64,
            printf: printf_ptr as u64,
            input: input_ptr as u64,
        });
    }
}

#[repr(C)]
struct TrainingParams {
    ddr_ctrl_base: u64,
    cs_num: u64,
    freq: u64,
    printf: u64,
    input: u64,
}

type TrainingEntry = unsafe extern "C" fn(*const TrainingParams);

unsafe fn training_entry(addr: usize) -> TrainingEntry {
    unsafe { core::mem::transmute(addr as *const ()) }
}
