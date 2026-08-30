//! Host-side flash library: USB state machine, postcard-rpc client, and GPT helpers.
//!
//! The tree is three layers: this client talks USB to the on-board flash-server,
//! which reuses SPL drivers. Both the client and the server depend on
//! `k1-musebook-spl`.

pub mod gpt;
pub mod rpc;
pub mod usb;

use self::{
    rpc::{FlashClient, NvmeReadBytesError, NvmeWriteBytesError},
    usb::{RpcReady, Usb},
};
