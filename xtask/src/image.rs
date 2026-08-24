//! minimal `objcopy -O binary`: concatenate ELF PT_LOAD segments by physical address
//! cargo objcopy --release --bin xxx -- --strip-all -O binary yyy.bin

use color_eyre::eyre::{Result, eyre};
use object::{
    Endianness,
    elf::{FileHeader64, PT_LOAD},
    read::elf::{FileHeader, ProgramHeader},
};

pub fn from_elf(elf: &[u8]) -> Result<Vec<u8>> {
    let header = FileHeader64::<Endianness>::parse(elf)?;
    let endian = header.endian()?;
    let segments = header
        .program_headers(endian, elf)?
        .iter()
        .filter(|ph| ph.p_type(endian) == PT_LOAD && ph.p_filesz(endian) > 0)
        .collect::<Vec<_>>();

    let base = segments
        .iter()
        .map(|ph| ph.p_paddr(endian))
        .min()
        .ok_or_else(|| eyre!("no loadable segment in ELF"))?;

    let mut image = Vec::new();
    for ph in segments {
        let data = ph
            .data(endian, elf)
            .map_err(|()| eyre!("segment data out of bounds"))?;
        let offset = (ph.p_paddr(endian) - base) as usize;
        image.resize(image.len().max(offset + data.len()), 0); // zero-fill gaps between segments
        image[offset..offset + data.len()].copy_from_slice(data);
    }
    Ok(image)
}
