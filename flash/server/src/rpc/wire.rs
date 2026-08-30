//! USB wire adapters for postcard-rpc.
//!
//! `UsbWireTx` copies a frame into `tx_buf` and, if the length is an exact
//! multiple of 512, appends one `0` so a ZLP is not mistaken for end-of-transfer.
//! `UsbWireRx` reads a 4-byte little-endian length, then the body.

use core::{convert::Infallible, fmt::Arguments};

use k1_musebook_spl::log;
use postcard_rpc::{
    Topic,
    header::{VarHeader, VarKeyKind},
    server::{WireRx, WireRxErrorKind, WireSpawn, WireTx, WireTxErrorKind},
    standard_icd::LoggingTopic,
};

use super::usb;

/// postcard-rpc TX that sends through BROM USB.
#[derive(Clone, Copy, Debug)]
pub struct UsbWireTx;

/// postcard-rpc RX that reads a length prefix, then the frame body.
#[derive(Default, Debug)]
pub struct UsbWireRx;

/// No-op spawn impl; all endpoints are blocking.
#[derive(Clone, Copy, Default)]
pub struct UsbWireSpawn;

impl WireSpawn for UsbWireSpawn {
    type Error = Infallible;
    type Info = ();
    fn info(&self) -> &Self::Info {
        &()
    }
}

impl UsbWireTx {
    /// Send `tx_buf[..len]`, appending a `0` when `len` is a multiple of 512.
    fn send_frame(len: usize) -> Result<(), WireTxErrorKind> {
        let buf = usb::tx_buf();
        let len = if len.is_multiple_of(usb::BROM_USB_MAX_PACKET_BYTES) {
            *buf.get_mut(len).ok_or(WireTxErrorKind::Other)? = 0;
            len + 1
        } else {
            len
        };
        usb::send(&buf[..len]).map_err(|_| WireTxErrorKind::ConnectionClosed)
    }
}

impl WireTx for UsbWireTx {
    type Error = WireTxErrorKind;

    async fn send<T: serde::Serialize + ?Sized>(
        &self,
        hdr: VarHeader,
        msg: &T,
    ) -> Result<(), Self::Error> {
        let buf = usb::tx_buf();
        let (hdr_used, rest) = hdr.write_to_slice(buf).ok_or(WireTxErrorKind::Other)?;
        let body = postcard::to_slice(msg, rest).map_err(|_| WireTxErrorKind::Other)?;
        let total = hdr_used.len() + body.len();
        Self::send_frame(total)
    }

    async fn send_raw(&self, frame: &[u8]) -> Result<(), Self::Error> {
        let buf = usb::tx_buf();
        let dst = buf.get_mut(..frame.len()).ok_or(WireTxErrorKind::Other)?;
        dst.copy_from_slice(frame);
        Self::send_frame(frame.len())
    }

    async fn send_log_str(&self, kkind: VarKeyKind, s: &str) -> Result<(), Self::Error> {
        let mut key = postcard_rpc::header::VarKey::Key8(LoggingTopic::TOPIC_KEY);
        key.shrink_to(kkind);
        let hdr = VarHeader {
            key,
            seq_no: postcard_rpc::header::VarSeq::Seq2(0),
        };
        self.send::<str>(hdr, s).await
    }

    async fn send_log_fmt<'a>(
        &self,
        _kkind: VarKeyKind,
        _a: Arguments<'a>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl WireRx for UsbWireRx {
    type Error = WireRxErrorKind;

    async fn receive<'a>(&mut self, buf: &'a mut [u8]) -> Result<&'a mut [u8], Self::Error> {
        loop {
            usb::receive(&mut buf[..usb::BROM_USB_MAX_PACKET_BYTES])
                .map_err(|_| WireRxErrorKind::ConnectionClosed)?;
            let len = u32::from_le_bytes(buf[..4].try_into().unwrap()) as usize;
            if len == 0 || len > buf.len() {
                log::warn!(
                    "wire rx: invalid frame len {len:#x}, prefix head {:02x?}, ignoring",
                    &buf[..8]
                );
                continue;
            }
            usb::receive(&mut buf[..len]).map_err(|_| WireRxErrorKind::ConnectionClosed)?;
            return Ok(&mut buf[..len]);
        }
    }
}
