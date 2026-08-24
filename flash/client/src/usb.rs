use std::{marker::PhantomData, path::Path, time::Duration};

use k1_musebook_flash_server::usb::{
    BROM_USB_MAX_PACKET_BYTES, K1_MUSEBOOK_PID, K1_MUSEBOOK_VID, TX_BUFFER_SIZE,
};
use nusb::{
    DeviceInfo, Endpoint,
    transfer::{Buffer, Bulk, In, Out, TransferError},
};
use postcard_rpc::{
    header::VarSeqKind,
    host_client::{HostClient, WireRx, WireSpawn, WireTx},
    standard_icd::WireError,
};
use tokio::{fs, time};

pub struct Usb<STAGE> {
    ep_in: Endpoint<Bulk, In>,
    ep_out: Endpoint<Bulk, Out>,
    stage: PhantomData<STAGE>,
}

pub struct Connected;
pub struct BromFastboot;
pub struct ImageSent;
pub struct ServerBooted;
pub struct RpcReady;

pub enum Stage {
    BromFastboot(Usb<BromFastboot>),
    FlashServer(Usb<RpcReady>),
}

impl Usb<Connected> {
    pub async fn connect_k1_musebook() -> Result<Self, ConnectK1MusebookError> {
        println!("connecting to K1 Musebook device ...");
        let info = Self::find_k1_musebook().await?;
        let interface = info
            .open()
            .await
            .map_err(ConnectK1MusebookError::OpenDevice)?
            .claim_interface(0)
            .await
            .map_err(ConnectK1MusebookError::ClaimInterface)?;
        let ep_in = interface
            .endpoint::<Bulk, In>(Self::EP_IN)
            .map_err(ConnectK1MusebookError::OpenEndpoint)?;
        let ep_out = interface
            .endpoint::<Bulk, Out>(Self::EP_OUT)
            .map_err(ConnectK1MusebookError::OpenEndpoint)?;

        println!("connected to K1 Musebook device");
        Ok(Self {
            ep_in,
            ep_out,
            stage: PhantomData,
        })
    }

    pub async fn detect_stage(mut self) -> Stage {
        loop {
            match time::timeout(Duration::from_secs(5), self.send_cmd("getvar:version")).await {
                Ok(Ok(resp)) if resp.starts_with(b"FLASH_SERVER") => {
                    return Stage::FlashServer(self.next_stage());
                }
                Ok(Ok(resp)) if resp.starts_with(b"OKAY") => {
                    return Stage::BromFastboot(self.next_stage());
                }
                _ => {
                    println!("neither flash server nor brom fastboot detected, retrying ...");
                    time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }
}

impl Usb<BromFastboot> {
    pub async fn send_flash_server_image(
        mut self,
        path: &Path,
    ) -> Result<Usb<ImageSent>, SendFlashServerImageError> {
        let image = fs::read(path).await?;
        let image_path = path.display();
        let image_size = image.len();
        println!("sending flash server image {image_path} ({image_size} bytes) ...",);

        self.send_cmd(&format!("download:{image_size:08X}"))
            .await?
            .starts_with(b"DATA")
            .ok_or(SendFlashServerImageError::BromRejectedDownload)?;
        self.bulk_write(image).await?;
        self.bulk_read()
            .await?
            .starts_with(b"OKAY")
            .ok_or(SendFlashServerImageError::BromRejectedDownload)?;

        println!("flash server image sent successfully");
        Ok(self.next_stage())
    }
}

impl Usb<ImageSent> {
    pub async fn boot_flash_server(mut self) -> Result<Usb<ServerBooted>, BootFlashServerError> {
        println!("booting flash server ...");
        self.send_cmd("continue")
            .await?
            .starts_with(b"OKAY")
            .ok_or(BootFlashServerError::BromContinueFailed)?;
        Ok(self.next_stage())
    }
}

impl Usb<ServerBooted> {
    pub async fn wait_usb_reenumerate(self) -> Result<Usb<RpcReady>, WaitReenumerateError> {
        println!("USB re-enumeration start");
        let start = tokio::time::Instant::now();
        println!("waiting for USB device to disappear ...");
        while start.elapsed() < Self::TIMEOUT {
            if Self::find_k1_musebook().await.is_err() {
                println!("USB device disappeared");
                break;
            }
            time::sleep(Duration::from_millis(50)).await;
        }
        println!("waiting for USB device to appear ...");
        while start.elapsed() < Self::TIMEOUT {
            if Self::find_k1_musebook().await.is_ok() {
                println!("USB device appeared");
                tokio::time::sleep(Duration::from_millis(300)).await;
                let reconnected = Usb::<Connected>::connect_k1_musebook().await?;
                return Ok(reconnected.next_stage());
            }
            time::sleep(Duration::from_millis(50)).await;
        }
        println!("USB re-enumeration timeout");
        Err(WaitReenumerateError::Timeout)
    }
}

impl Usb<RpcReady> {
    pub fn client(self) -> HostClient<WireError> {
        HostClient::new_with_wire(
            NusbWireTx(self.ep_out),
            NusbWireRx(self.ep_in),
            NusbSpawn,
            VarSeqKind::Seq2,
            "error",
            8,
        )
    }
}

impl<T> Usb<T> {
    const EP_OUT: u8 = 0x02;
    const EP_IN: u8 = 0x81;
    const TIMEOUT: Duration = Duration::from_secs(30);

    async fn find_k1_musebook() -> Result<DeviceInfo, FindK1MusebookError> {
        nusb::list_devices()
            .await
            .map_err(FindK1MusebookError::ListDevices)?
            .filter(|info| info.vendor_id() == K1_MUSEBOOK_VID)
            .find(|info| info.product_id() == K1_MUSEBOOK_PID)
            .ok_or(FindK1MusebookError::NotFound)
    }

    fn next_stage<U>(self) -> Usb<U> {
        Usb {
            ep_in: self.ep_in,
            ep_out: self.ep_out,
            stage: PhantomData,
        }
    }

    async fn send_cmd(&mut self, cmd: &str) -> Result<Vec<u8>, TransferError> {
        self.bulk_write(cmd.as_bytes()).await?;
        self.bulk_read().await
    }

    async fn bulk_write(&mut self, data: impl Into<Buffer>) -> Result<(), TransferError> {
        self.ep_out.submit(data.into());
        self.ep_out.next_complete().await.status
    }

    async fn bulk_read(&mut self) -> Result<Vec<u8>, TransferError> {
        self.ep_in.submit(Buffer::new(self.ep_in.max_packet_size()));
        self.ep_in
            .next_complete()
            .await
            .into_result()
            .map(Buffer::into_vec)
    }
}

struct NusbWireTx(Endpoint<Bulk, Out>);
impl WireTx for NusbWireTx {
    type Error = TransferError;

    async fn send(&mut self, data: Vec<u8>) -> Result<(), Self::Error> {
        let mut prefix = vec![0u8; BROM_USB_MAX_PACKET_BYTES];
        prefix[..4].copy_from_slice(&(data.len() as u32).to_le_bytes());

        self.0.submit(prefix.into());
        self.0.submit(data.into());

        self.0.next_complete().await.status?;
        self.0.next_complete().await.status
    }
}

const MAX_TRANSFER_BYTES: usize = TX_BUFFER_SIZE;
const IN_FLIGHT_REQS: usize = 4;

struct NusbWireRx(Endpoint<Bulk, In>);
impl WireRx for NusbWireRx {
    type Error = TransferError;

    async fn receive(&mut self) -> Result<Vec<u8>, Self::Error> {
        while self.0.pending() < IN_FLIGHT_REQS {
            self.0.submit(Buffer::new(MAX_TRANSFER_BYTES));
        }
        self.0
            .next_complete()
            .await
            .into_result()
            .map(Buffer::into_vec)
    }
}

struct NusbSpawn;

impl WireSpawn for NusbSpawn {
    fn spawn(&mut self, fut: impl Future<Output = ()> + Send + 'static) {
        drop(tokio::task::spawn(fut));
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ConnectK1MusebookError {
    #[error("find K1 Musebook device failed: {0:?}")]
    FindK1Musebook(#[from] FindK1MusebookError),
    #[error("open device failed: {0:?}")]
    OpenDevice(nusb::Error),
    #[error("claim interface failed: {0:?}")]
    ClaimInterface(nusb::Error),
    #[error("open endpoint failed: {0:?}")]
    OpenEndpoint(nusb::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum FindK1MusebookError {
    #[error("list devices failed: {0:?}")]
    ListDevices(nusb::Error),
    #[error("K1 Musebook device not found (VID=361c PID=1001)")]
    NotFound,
}

#[derive(thiserror::Error, Debug)]
pub enum SendFlashServerImageError {
    #[error("read image file failed: {0:?}")]
    ReadImage(#[from] std::io::Error),
    #[error("bulk transfer failed: {0:?}")]
    BulkTransfer(#[from] TransferError),
    #[error("BROM rejected download")]
    BromRejectedDownload,
}

#[derive(thiserror::Error, Debug)]
pub enum BootFlashServerError {
    #[error("bulk transfer failed: {0:?}")]
    BulkTransfer(#[from] TransferError),
    #[error("BROM continue failed")]
    BromContinueFailed,
}

#[derive(thiserror::Error, Debug)]
pub enum WaitReenumerateError {
    #[error("reconnect K1 Musebook device failed: {0:?}")]
    ReconnectError(#[from] ConnectK1MusebookError),
    #[error("USB re-enumerate timeout")]
    Timeout,
}
