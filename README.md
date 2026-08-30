# K1 MUSE Book FSBL

A secondary program loader (SPL / FSBL) for the SpacemiT K1 MUSE Book written in Rust,
together with the USB flashing toolchain that goes with it.

```
BROM → k1-musebook-fsbl → SBI → Kernel
```

---

- [Boot chain](#boot-chain)
- [Quick start](#quick-start)
- [Command reference](#command-reference)
- [Project structure](#project-structure)
- [Address map](#address-map)
- [Adjusting the layout](#adjusting-the-layout)
- [Image formats](#image-formats)
- [References](#references)

---

## Boot chain

```
BROM (on-chip ROM, immutable)
  │ reads the 80-byte bootinfo at NOR 0x00000 for spl0_offset / spl1_offset / spl_size_limit
  │ reads the FSBL from NOR 0x20000, verifies the AIHD header and the RSA-2048 signature
  │ copies it into SRAM 0xC0800000 and jumps to 0xC0801000
  ▼
k1-musebook-spl (runs in SRAM, about 50 KiB)
  │ hart 0:
  │   1. set up the stack, clear BSS
  │   2. UART log, M-mode trap handler, Generic Counter
  │   3. I2C8 → SPM8821 raises VDD_CORE to 1.05 V
  │   4. PLL3 → both clusters up to 1600 MHz
  │   5. LPDDR4X two-pass init + stepwise 1200 / 1600 / 2400 MT training + pattern self-check
  │   6. enable D/I-cache, BPU, prefetch, L2 snoop
  │   7. bring PCIe Port C up as RC (PUPHY → Gen2 LTSSM L0 → iATU → NVMe BAR)
  │   8. parse the GPT, read sbi / kernel / dtb / initramfs into DDR by name
  │   9. join both clusters to the CCI coherency domain, wake harts 1-7
  │ harts 1-7: spin until hart 0 is done, then each enables cache and performance features
  │
  │ every hart jumps to SBI: a0 = hartid, a1 = DTB address, a2 = &fw_dynamic_info
  ▼
SBI (DDR 0x0008_0000)
  ▼
Kernel (DDR 0x0020_0000, S-mode)
```

---

## Quick start

### Prerequisites

Only the [Rust toolchain](https://rust-lang.org/tools/install/) is required.

USB access on Linux needs a udev rule or `sudo` (the device VID:PID is `361c:1001`).

### Build

```sh
cargo xtask build
```

The artifacts are written to `images/`:

| File | Description |
| --- | --- |
| `images/k1-musebook-spl-fsbl.bin` | Signed SPL, flashed to NOR `0x20000` |
| `images/k1-musebook-flash-server-fsbl.bin` | Signed on-board flash service, downloaded and run temporarily by BROM fastboot |
| `images/bootinfo.bin` | 80-byte bootinfo, flashed to NOR `0x0` |

### Flashing

1. Connect the host to the OTG port on the left side of the MUSE Book over USB.
2. Hold a SIM-eject pin down in the download-mode pinhole on the right side of the MUSE Book while pressing the power button.
3. The MUSE Book should now enter BROM fastboot mode (`361c:1001` appears on USB).
4. Run the flashing commands you need on the host:

```sh
# NOR @ 0x0: flash the bootinfo
cargo xtask bootinfo flash

# NOR @ 0x20000: flash the SPL
cargo xtask flash nor flash

# SSD: create the GPT partition table from spl/src/layout.rs
cargo xtask flash gpt init

# SSD: flash the images for each stage
cargo xtask flash gpt flash \
    sbi=./sbi.bin \
    kernel=./kernel.bin \
    dtb=./dtb.bin \
    initramfs=./initramfs.cpio.gz \
    rootfs=./rootfs.ext2

# Confirm the GPT partition table
cargo xtask flash gpt list
```

---

## Command reference

### `cargo xtask`

| Command | Effect |
| --- | --- |
| `cargo xtask build` | Build the SPL and the flash-server → extract `PT_LOAD` → RSA-sign → write `images/*-fsbl.bin`; also pack `images/bootinfo.bin` |
| `cargo xtask bootinfo flash [CONFIG]` | Pack the bootinfo from `CONFIG` (default `bootinfo.toml`) and write it to NOR `0x0` |
| `cargo xtask bootinfo read [OUT]` | Read the 80 bytes back from NOR `0x0` and decode them into TOML at `OUT` (default `./bootinfo-out.toml`; the raw `./bootinfo-out.bin` is kept as well) |
| `cargo xtask flash <ARGS…>` | Build the flash-server, then forward `ARGS` verbatim to the host CLI |

### flash CLI

The subcommands below go after `cargo xtask flash`.

The global `--server-image <PATH>` option selects the on-board image that is uploaded and then talks
to the host; it defaults to `./images/k1-musebook-flash-server-fsbl.bin`.

| Command | Description |
| --- | --- |
| `ping` | Handshake; prints the ICD version of the flash-server (`0x00010000`) |
| `nor flash [OFFSET] [FILE]` | Erase the 4 KiB-aligned window the write covers, then program it. Defaults are `OFFSET=0x20000` and `FILE=./images/k1-musebook-spl-fsbl.bin` |
| `nor read <OFFSET> <LEN> [OUT]` | Read NOR; writes to `./nor-read-out.bin` by default |
| `nvme flash <LBA> <FILE>` | Write a file starting at the given LBA (the tail is padded with `0xFF` to a 512-byte boundary) |
| `nvme read <LBA> <LEN> <OUT>` | Read `LEN` bytes starting at the given LBA |
| `gpt list` | Parse the primary GPT and list the name / start and end LBA / size of every partition |
| `gpt init [--disk-lba-count N]` | Write a protective MBR + primary GPT + backup GPT from `spl/src/layout.rs`. The disk capacity is inferred from an existing backup header when possible; pass `--disk-lba-count` when it cannot be inferred |
| `gpt flash NAME=FILE …` | Write files into partitions by name |

---

## Project structure

| Crate | Target | Role |
| --- | --- | --- |
| `spl` (`k1-musebook-spl`) | RISC-V firmware + library | The SPL itself; also exports every hardware driver plus `layout.rs` and `gpt.rs` as a library |
| `flash/server` (`k1-musebook-flash-server`) | RISC-V firmware | On-board flash service; reuses the DDR / PCIe / NVMe drivers from `spl` and adds QSPI NOR and USB of its own |
| `flash/client` (`k1-musebook-flash-client`) | Host | Flashing CLI; depends on `spl` (shared partition layout) and `flash/server` (shared RPC ICD) |
| `xtask` | Host | Build, ELF-to-raw-image conversion, RSA signing, bootinfo packing, and flashing orchestration |


### SPL modules (`spl/src/`)

| File | Role |
| --- | --- |
| `main.rs` | Reset entry, hart dispatch, BSS clear, the `boot()` sequence, the jump to SBI, panic handler |
| `lib.rs` | Module list and the single place where cross-module `use`s are gathered |
| `layout.rs` | GPT partition table, DDR load addresses, NVMe DMA window |
| `mmio.rs` | `MMIO<T>` (treats a base address as a typed register block) and `Raw` (reads and writes a `u32` at a byte offset) |
| `log.rs` / `uart.rs` | UART backend for the `log` crate; polled 16550 transmit with a CR inserted before every newline |
| `trap.rs` | Masks all interrupts, installs a Direct-mode `mtvec`; any trap prints `mcause`/`mepc`/`mtval` and panics |
| `time.rs` | Generic Counter (`0xD5001000`) and a busy-wait `sleep` |
| `pinmux.rs` | MFPR (`0xD401E000`): the QSPI and I2C pins this firmware uses |
| `pcr.rs` | Power / clock / reset register blocks: `APMU` / `APBC` / `APBS` / `MPMU` |
| `i2c.rs` | TWSI8 (`0xD401D800`) 7-bit-address master write |
| `cci.rs` | CCI snoop / DVM, cross-cluster cache coherency |
| `cpu/` | Voltage (`voltage`), frequency (`freq`), cache (`cache`), BPU/prefetch/snoop (`perf`), secondary-hart wake (`multicore`), X60 custom CSRs (`csr`) |
| `ddr/` | LPDDR4X bring-up: `clock` / `ctrl` / `phy` / `dfi` / `dram` / `byte` / `freq` / `train` / `image` (training firmware blob) / `verify` |
| `pcie/` | Port C as RC: `clock` / `phy` / `link` (Gen2 LTSSM) / `atu` (iATU windows) / `bar` |
| `nvme/` | Admin + I/O queue setup, 4 KiB chunked read and write, DMA cache maintenance |
| `gpt.rs` | Parses the GPT from LBA 1, indexes partitions by UTF-16 name, `cbo.clean` after loading into DDR |
| `handoff.rs` | OpenSBI `fw_dynamic_info` v2 structure |

---

## Address map

### SSD GPT partitions and DDR load addresses

All of it is defined in [`spl/src/layout.rs`](spl/src/layout.rs).

| Partition | Start LBA | Partition size | DDR load address | DDR window cap |
| --- | --- | --- | --- | --- |
| `sbi` | 2048 | 512 KiB | `0x0008_0000` | 1 MiB |
| `kernel` | 4096 | 12 MiB | `0x0020_0000` | 14 MiB |
| `dtb` | 28672 | 256 KiB | `0x0100_0000` | 1 MiB |
| `initramfs` | 32768 | 64 MiB | `0x0800_0000` | 64 MiB |
| `rootfs` | 163840 | rest of the disk | not loaded | — |


Two more DDR regions serve non-partition purposes:

| Address | Purpose |
| --- | --- |
| `0x0001_0000` | Pattern self-check buffer used once DDR training finishes (512 bytes) |
| `0x0400_0000` | NVMe DMA window (28 KiB): admin SQ/CQ, I/O SQ/CQ, read and write PRP1/PRP2 |

The flash-server uses additional DDR at runtime:

| Address | Purpose |
| --- | --- |
| `0x0500_0000` | USB RX buffer (1 MiB + 4 KiB) |
| `0x0510_1000` | USB TX buffer (1 MiB + 4 KiB) |
| `0x1000_0000` | DDR stack top switched to before entering the RPC listen loop |

### PCIe address windows

| Window | CPU address | Size | Description |
| --- | --- | --- | --- |
| CFG | `0xA000_0000` | 1 MiB | Outbound iATU region 0, TLP type CFG0, target bus 1 dev 0 fn 0 |
| MEM | `0xA200_0000` | 352 MiB | Outbound iATU region 1, 1:1 pass-through; NVMe BAR0 points here |

The MMIO base of the NVMe controller registers is therefore `0xA200_0000` (`NVME_CTRL_BASE`), with
the doorbells at `+0x1000`.

### SRAM

The K1 has only 256 KiB of SRAM (`0xC0800000`–`0xC0840000`), and each of the two firmwares has its
own linker script.

**SPL** ([`spl/linker-script.spl.ld`](spl/linker-script.spl.ld)):

| Range | Purpose |
| --- | --- |
| `0xC080_0000`–`0xC080_1000` | FSBL header (4 KiB, generated by `xtask`, not part of the linker script) |
| `0xC080_1000`–`0xC083_4000` | `.text` / `.rodata` / `.data` |
| `0xC083_7000`–`0xC083_9000` | `.bss` (8 KiB) |
| `0xC083_9000`–`0xC084_0000` | Stack (28 KiB, `STACK_TOP = 0xC0840000`, grows down) |

**flash-server** ([`flash/server/linker-script.flash.ld`](flash/server/linker-script.flash.ld)):

Because it reuses the USB ROM routines of the BROM, it has to keep clear of both the training
firmware area and the BROM's own globals / stack (`0xC083_8000`–`0xC084_0000`), so everything is
pushed further down:

| Range | Purpose |
| --- | --- |
| `0xC080_0000`–`0xC080_1000` | FSBL header |
| `0xC080_1000`–`0xC082_1000` | `.text` / `.rodata` / `.data` (128 KiB) |
| `0xC082_1000`–`0xC082_3000` | `.bss` (8 KiB) |
| `0xC082_3000`–`0xC083_2000` | Stack (60 KiB, `STACK_TOP = 0xC0832000`) |

The 1 MiB USB RX/TX buffers do not fit into SRAM, so the flash-server moves `sp` to DDR before
entering the RPC loop.

### QSPI NOR

Described by [`bootinfo.toml`](bootinfo.toml); the capacity is 1 MiB:

| Offset | Contents |
| --- | --- |
| `0x00000` | bootinfo (80 bytes, parsed by BROM directly) |
| `0x20000` | Primary FSBL (`spl0_offset`) |
| `0x70000` | Backup FSBL (`spl1_offset`; BROM falls back here when the primary slot fails verification) |

Once the SPL is running, NOR takes no further part in loading data.

---

## Adjusting the layout

### Changing the partition layout or the load addresses

Edit [`spl/src/layout.rs`](spl/src/layout.rs).

```rust
pub const KERNEL: GptPart = GptPart {
    name: "kernel",        // GPT partition name (matched as UTF-16)
    lba_start: 4096,       // first LBA on disk
    lba_max: 24576,        // number of LBAs the partition occupies
    load_base: 0x0020_0000, // address it is copied to in DDR
    load_max: 0x00E0_0000,  // cap of that DDR window
};
```

### Changing the NOR layout

Edit [`bootinfo.toml`](bootinfo.toml), then run `cargo xtask bootinfo flash`.
`spl0_offset` / `spl1_offset` are validated so that they do not overlap the first sector holding the
bootinfo itself, and so that the two slots do not overlap each other.

### Changing the SRAM layout

Edit the corresponding linker script. Mind the two hard constraints: the training firmware area, and
the BROM region the flash-server must stay clear of.

---

## Image formats

### FSBL

Produced by [`xtask/src/fsbl.rs`](xtask/src/fsbl.rs): a 4 KiB header + the 32-byte-aligned raw image + a 256-byte signature.

| Offset | Length | Contents |
| --- | --- | --- |
| `0x000` | 256 | ROTPK (RSA-2048 public modulus, big-endian) |
| `0x100` | 32 | header0 (`AIHD` magic + version=1 + certificate area length `0x1000`) |
| `0x120` | 480 | keydata (four key tables: `spl`/`uboot`/`kernel`/`rootfs` + SHA-256 of the ROTPK) |
| `0x300` | 2048 | oem_key (slot 0 holds the signing public modulus, the rest is zero) |
| `0xB00` | 256 | signature0 = RSA-PKCS#1v1.5-SHA256(header0 ‖ keydata ‖ oem_key) |
| `0xC00` | 992 | padding |
| `0xFE0` | 32 | header1 (`AIHD` + raw image length) |
| `0x1000` | — | Raw image (the `PT_LOAD` segments of the ELF concatenated by physical address, holes zero-filled) |
| tail | 256 | signature1 = RSA-PKCS#1v1.5-SHA256(header1 ‖ raw image) |

### bootinfo

Produced by [`xtask/src/bootinfo.rs`](xtask/src/bootinfo.rs), `0x50` bytes in total: a `0x40`-byte header + CRC32 + 12 bytes of padding.

| Offset | Field | Description |
| --- | --- | --- |
| `0x00` | magic | `0xB00714F0` |
| `0x04` | version | `0x00010001` |
| `0x08` | flash_type | `NORF` |
| `0x10` | `page_size` | NOR page size (256) |
| `0x14` | `block_size` | NOR erase block size (`0x10000`) |
| `0x18` | `total_size` | NOR capacity (`0x100000`) |
| `0x20` | `spl0_offset` | Primary FSBL offset (`0x20000`) |
| `0x24` | `spl1_offset` | Backup FSBL offset (`0x70000`) |
| `0x28` | `spl_size_limit` | Largest FSBL size the BROM allows (`0x36000`) |
| `0x2C` | `partitiontable0_offset` | Unused, kept for the BROM layout |
| `0x30` | `partitiontable1_offset` | Unused, kept for the BROM layout |
| `0x40` | crc32 | IEEE CRC32 over the first `0x40` bytes |

### Flashing protocol

Host and board share the `postcard-rpc` ICD in [`flash/server/src/protocol.rs`](flash/server/src/protocol.rs):

| Endpoint | Path | Request / Response |
| --- | --- | --- |
| `PingEndpoint` | `ping` | `()` → `u32` (version `0x00010000`) |
| `NorEraseEndpoint` | `nor/erase` | `NorRange` → `Result<(), FlashServerError>` |
| `NorWriteEndpoint` | `nor/write` | `NorChunk` → `Result<(), _>` |
| `NorReadEndpoint` | `nor/read` | `NorRange` → `Result<ByteBuf, _>` |
| `NvmeWriteEndpoint` | `nvme/write` | `NvmeChunk` → `Result<(), _>` |
| `NvmeReadEndpoint` | `nvme/read` | `NvmeRange` → `Result<ByteBuf, _>` |

A single payload is capped at 1 MiB. Underneath it uses the USB bulk endpoints the BROM has already
enumerated (OUT `0x02` / IN `0x81`); ahead of every RPC frame the host sends one 512-byte packet
whose first 4 bytes are the frame length in little endian.

The board side does not implement a USB driver of its own, it calls the RX/TX routines in the BROM
ROM directly (`0xFFE037B6` / `0xFFE038D0`). The price is that the BROM's `gp` and global area have to
stay intact — which is why the first thing the flash-server entry does is restore
`gp = 0xC0838C10`, and why the linker script has to stay clear of everything above `0xC0838000`.
DDR init also disrupts the USB controller, so `usb::init()` clears the BROM's `g_usb_ready` and
re-runs `controller_run` to trigger re-enumeration; the host-side `wait_usb_reenumerate` is the
matching half.

---

## References

- [Official SpacemiT K1 BSP uboot-2022.10](https://github.com/spacemit-com/uboot-2022.10)
- [SpacemiT K1 chip documentation](https://github.com/spacemit-com/docs-chip)
