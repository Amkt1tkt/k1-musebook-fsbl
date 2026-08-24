use std::path::PathBuf;

use clap::{Parser, Subcommand};
use k1_musebook_flash_client::{
    rpc::FlashClient,
    usb::{Stage, Usb},
};
use tokio::fs;

const VERSION: &str = const_format::formatcp!("{:#X}", k1_musebook_flash_server::protocol::VERSION);

#[derive(Parser)]
#[command(
    name = "k1-musebook-flash",
    version = VERSION,
    about = "SpacemiT K1 MUSE Book firmware flash tool"
)]
struct Cli {
    #[arg(
        long,
        global = true,
        default_value = "./images/k1-musebook-flash-server-fsbl.bin"
    )]
    server_image: PathBuf,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    Ping,
    Nor {
        #[command(subcommand)]
        cmd: CmdNor,
    },
    Nvme {
        #[command(subcommand)]
        cmd: CmdNvme,
    },
    Gpt {
        #[command(subcommand)]
        cmd: CmdGpt,
    },
}

#[derive(Subcommand)]
enum CmdNor {
    Flash {
        #[arg(
            value_parser = parse_u64,
            default_value_t = 0x2_0000,
        )]
        offset: u64,
        #[arg(default_value = "./images/k1-musebook-spl-fsbl.bin")]
        file: PathBuf,
    },
    Read {
        #[arg(value_parser = parse_u64)]
        offset: u64,
        #[arg(value_parser = parse_u64)]
        len: u64,
        #[arg(default_value = "./nor-read-out.bin")]
        out: PathBuf,
    },
}

#[derive(Subcommand)]
enum CmdNvme {
    Flash {
        #[arg(value_parser = parse_u64)]
        lba: u64,
        file: PathBuf,
    },
    Read {
        #[arg(value_parser = parse_u64)]
        lba: u64,
        #[arg(value_parser = parse_u64)]
        len: u64,
        out: PathBuf,
    },
}

#[derive(Subcommand)]
enum CmdGpt {
    List,
    Init {
        #[arg(long)]
        disk_lba_count: Option<u64>,
    },
    Flash {
        #[arg(value_parser = parse_partitions, required = true, help = "NAME=FILE NAME=FILE ...")]
        parts: Vec<(String, PathBuf)>,
    },
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();

    let usb = match Usb::connect_k1_musebook().await?.detect_stage().await {
        Stage::FlashServer(usb) => usb,
        Stage::BromFastboot(usb) => {
            usb.send_flash_server_image(&cli.server_image)
                .await?
                .boot_flash_server()
                .await?
                .wait_usb_reenumerate()
                .await?
        }
    };

    let client = FlashClient::connect(usb).await?;

    match cli.cmd {
        Cmd::Ping => {
            let version = client.ping().await?;
            println!("pong version={version:#010x}");
        }
        Cmd::Nor { cmd } => match cmd {
            CmdNor::Flash { offset, file } => {
                client.nor_write_file(offset as u32, &file).await?;
            }
            CmdNor::Read { offset, len, out } => {
                let data = client.nor_read(offset as u32, len as u32).await?;
                fs::write(&out, &data).await?;
                println!("wrote {} bytes to {}", data.len(), out.display());
            }
        },
        Cmd::Nvme { cmd } => match cmd {
            CmdNvme::Flash { lba, file } => {
                client.nvme_write_file(lba, &file).await?;
            }
            CmdNvme::Read { lba, len, out } => {
                let data = client.nvme_read(lba, len as u32).await?;
                fs::write(&out, &data).await?;
                println!("wrote {} bytes to {}", data.len(), out.display());
            }
        },
        Cmd::Gpt { cmd } => match cmd {
            CmdGpt::List => {
                client.gpt_list().await?;
            }
            CmdGpt::Init { disk_lba_count } => {
                client.gpt_init(disk_lba_count).await?;
            }
            CmdGpt::Flash { parts } => {
                client.gpt_flash(&parts).await?;
            }
        },
    }
    Ok(())
}

fn parse_partitions(s: &str) -> Result<(String, PathBuf), String> {
    let (name, file) = s
        .split_once('=')
        .ok_or_else(|| format!("expected NAME=FILE, got `{s}`"))?;
    Ok((name.to_string(), PathBuf::from(file)))
}

fn parse_u64(s: &str) -> color_eyre::Result<u64> {
    let s = s.to_ascii_lowercase();
    let s = s.trim();
    if let Some(h) = s.strip_prefix("0x") {
        Ok(u64::from_str_radix(h, 16)?)
    } else {
        Ok(s.parse()?)
    }
}
