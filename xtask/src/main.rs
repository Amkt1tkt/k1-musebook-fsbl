//! Host build/flash helper for K1 MUSE Book firmware.
//!
//! `build` compiles SPL and flash-server, extracts `PT_LOAD`, RSA-signs
//! `*-fsbl.bin`, and writes `bootinfo.bin`. `flash` builds the server then execs
//! flash-client. `bootinfo flash` / `bootinfo read` program or dump NOR offset 0.
//! BROM rejects images larger than `0x36000`.

mod bootinfo;
mod fsbl;
mod image;

use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use clap::{Parser, Subcommand};
use color_eyre::eyre::{Result, ensure};

/// RISC-V firmware rustc target triple.
const FIRMWARE_TARGET: &str = "riscv64gc-unknown-none-elf";
/// Host flash-client package invoked by `flash` and `bootinfo`.
const FLASH_CLIENT: &str = "k1-musebook-flash-client";
/// BROM maximum FSBL image size (`spl_size_limit`).
const BROM_FSBL_LIMIT: usize = 0x3_6000; // BROM spl_size_limit
const IMAGES_DIR: &str = "images";
const BOOTINFO_BIN: &str = "bootinfo.bin";
const BOOTINFO_CONFIG: &str = "bootinfo.toml";

#[derive(Parser)]
#[command(
    name = "cargo xtask",
    about = "SpacemiT K1 MUSE Book firmware build tool"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    Build,
    Flash {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    Bootinfo {
        #[command(subcommand)]
        cmd: Bootinfo,
    },
}

#[derive(Subcommand)]
enum Bootinfo {
    Flash {
        #[arg(default_value = BOOTINFO_CONFIG)]
        config: PathBuf,
    },
    Read {
        #[arg(default_value = "./bootinfo-out.toml")]
        out: PathBuf,
    },
}

/// Dispatch `build`, `flash`, or `bootinfo`.
fn main() -> Result<()> {
    color_eyre::install()?;

    match Cli::parse().cmd {
        Cmd::Build => {
            println!("Building SPL, flash server, and bootinfo...");
            build_fsbl("k1-musebook-spl")?;
            build_fsbl("k1-musebook-flash-server")?;
            build_bootinfo(&workspace_root().join(BOOTINFO_CONFIG))?;
        }
        Cmd::Flash { args } => {
            build_fsbl("k1-musebook-flash-server")?;
            run_flash_client(args)?;
        }
        Cmd::Bootinfo { cmd } => {
            build_fsbl("k1-musebook-flash-server")?;
            match cmd {
                Bootinfo::Flash { config } => {
                    build_bootinfo(&config)?;
                    run_flash_client(vec![
                        "nor".into(),
                        "flash".into(),
                        "0x0".into(),
                        format!("./{IMAGES_DIR}/{BOOTINFO_BIN}"),
                    ])?;
                }
                Bootinfo::Read { out } => {
                    read_bootinfo(&out)?;
                }
            }
        }
    }
    Ok(())
}

/// Read NOR offset 0 via flash-client and write a TOML dump next to the raw bin.
fn read_bootinfo(out: &Path) -> Result<()> {
    let out = workspace_path(out);
    let bin = out.with_extension("bin");
    run_flash_client(vec![
        "nor".into(),
        "read".into(),
        "0x0".into(),
        format!("{:#x}", bootinfo::BYTES),
        client_path(&bin),
    ])?;

    let info = bootinfo::Bootinfo::from_bytes(&fs::read(&bin)?)?;
    fs::write(&out, info.to_toml())?;
    println!("{}: {} ({})", out.display(), info.brief(), bin.display());
    Ok(())
}

/// Resolve `path` against the workspace root when it is relative.
fn workspace_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        workspace_root().join(path.strip_prefix(".").unwrap_or(path))
    }
}

/// Path string for flash-client: workspace-relative with a `./` prefix when possible.
fn client_path(path: &Path) -> String {
    path.strip_prefix(workspace_root())
        .map(|p| format!("./{}", p.display()))
        .unwrap_or_else(|_| path.display().to_string())
}

/// Pack `bootinfo.toml` (or `config`) into `images/bootinfo.bin`.
fn build_bootinfo(config: &Path) -> Result<PathBuf> {
    let info = bootinfo::Bootinfo::load(config)?;
    let bytes = info.to_bytes();

    let images = workspace_root().join(IMAGES_DIR);
    fs::create_dir_all(&images)?;
    let path = images.join(BOOTINFO_BIN);
    fs::write(&path, bytes)?;

    println!(
        "{}: {} bytes ({})",
        path.display(),
        bytes.len(),
        info.brief()
    );
    Ok(path)
}

/// cargo build → extract PT_LOAD → RSA-sign as `images/{package}-fsbl.bin`.
fn build_fsbl(package: &str) -> Result<PathBuf> {
    let elf = build_firmware_elf(package)?;

    let raw = image::from_elf(&fs::read(&elf)?)?;
    let key = fsbl::load_or_generate_key(&workspace_root().join(fsbl::KEY_FILE))?;
    let fsbl = fsbl::wrap(&raw, &key)?;

    let images = workspace_root().join(IMAGES_DIR);
    fs::create_dir_all(&images)?;
    let fsbl_path = images.join(format!("{package}-fsbl.bin"));
    fs::write(&fsbl_path, &fsbl)?;

    println!(
        "{}: {} bytes (raw {} bytes, limit {BROM_FSBL_LIMIT})",
        fsbl_path.display(),
        fsbl.len(),
        raw.len()
    );
    ensure!(
        fsbl.len() <= BROM_FSBL_LIMIT,
        "{} exceeds the BROM spl_size_limit ({BROM_FSBL_LIMIT} bytes)",
        fsbl_path.display()
    );
    Ok(fsbl_path)
}

/// `cargo build --release` the firmware package for [`FIRMWARE_TARGET`].
fn build_firmware_elf(package: &str) -> Result<PathBuf> {
    let args = [
        "build",
        "--release",
        "--target",
        FIRMWARE_TARGET,
        "--package",
    ];
    run(Command::new(cargo()).args(args).arg(package))?;
    Ok(workspace_root()
        .join("target")
        .join(FIRMWARE_TARGET)
        .join("release")
        .join(package))
}

/// `cargo run --release` the flash-client with `args` after `--`.
fn run_flash_client(args: Vec<String>) -> Result<()> {
    let cargo_args = ["run", "--release", "--package", FLASH_CLIENT, "--"];
    run(Command::new(cargo()).args(cargo_args).args(args))
}

/// Run `cmd` in the workspace root and fail if it exits non-zero.
fn run(cmd: &mut Command) -> Result<()> {
    println!("==> {}", format!("{cmd:?}").replace('"', ""));
    let status = cmd.current_dir(workspace_root()).status()?;
    ensure!(status.success(), "command failed with {status}");
    Ok(())
}

/// `CARGO` env or the `cargo` on `PATH`.
fn cargo() -> PathBuf {
    env::var_os("CARGO").map_or(PathBuf::from("cargo"), PathBuf::from)
}

/// Workspace root (parent of the `xtask` crate).
fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap()
}
