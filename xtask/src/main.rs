mod fsbl;
mod image;

use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use clap::{Parser, Subcommand};
use color_eyre::eyre::{Result, ensure};

const FIRMWARE_TARGET: &str = "riscv64gc-unknown-none-elf";
const FLASH_CLIENT: &str = "k1-musebook-flash-client";
const BROM_FSBL_LIMIT: usize = 0x3_6000; // BROM spl_size_limit
const IMAGES_DIR: &str = "images";

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
}

fn main() -> Result<()> {
    color_eyre::install()?;

    match Cli::parse().cmd {
        Cmd::Build => {
            println!("Building SPL and flash server...");
            build_fsbl("k1-musebook-spl")?;
            build_fsbl("k1-musebook-flash-server")?;
        }
        Cmd::Flash { args } => {
            build_fsbl("k1-musebook-flash-server")?;
            run_flash_client(args)?;
        }
    }
    Ok(())
}

/// cargo build → extract raw image → sign and pack as *-fsbl.bin, returns the FSBL path
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

fn run_flash_client(args: Vec<String>) -> Result<()> {
    let cargo_args = ["run", "--release", "--package", FLASH_CLIENT, "--"];
    run(Command::new(cargo()).args(cargo_args).args(args))
}

fn run(cmd: &mut Command) -> Result<()> {
    println!("==> {}", format!("{cmd:?}").replace('"', ""));
    let status = cmd.current_dir(workspace_root()).status()?;
    ensure!(status.success(), "command failed with {status}");
    Ok(())
}

fn cargo() -> PathBuf {
    env::var_os("CARGO").map_or(PathBuf::from("cargo"), PathBuf::from)
}

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap()
}
