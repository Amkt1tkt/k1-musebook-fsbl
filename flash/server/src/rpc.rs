use core::{
    future::Future,
    task::{Context as TaskContext, Poll, RawWaker, RawWakerVTable, Waker},
};

use k1_musebook_spl::{log, nvme::Nvme};
use postcard_rpc::{
    define_dispatch,
    server::{Dispatch, Server, SpawnContext},
};

use super::{
    ByteBuf, CHUNK_BYTES, ENDPOINT_LIST, FlashServerError, NorChunk, NorEraseEndpoint, NorRange,
    NorReadEndpoint, NorWriteEndpoint, NvmeChunk, NvmeRange, NvmeReadEndpoint, NvmeWriteEndpoint,
    PingEndpoint, ReadResult, TOPICS_IN_LIST, TOPICS_OUT_LIST, VERSION, WriteResult, nor, usb,
};

mod handler;
mod wire;

use self::{
    handler::{nor_erase, nor_read, nor_write, nvme_read, nvme_write, ping},
    wire::{UsbWireRx, UsbWireSpawn, UsbWireTx},
};

define_dispatch! {
    app: FlashApp;
    spawn_fn: spawn_fn;
    tx_impl: UsbWireTx;
    spawn_impl: UsbWireSpawn;
    context: Context;

    endpoints: {
        list: ENDPOINT_LIST;

        | EndpointTy        | kind     | handler    |
        | ----------------- | -------- | ---------- |
        | PingEndpoint      | blocking | ping       |
        | NorEraseEndpoint  | blocking | nor_erase  |
        | NorWriteEndpoint  | blocking | nor_write  |
        | NorReadEndpoint   | blocking | nor_read   |
        | NvmeWriteEndpoint | blocking | nvme_write |
        | NvmeReadEndpoint  | blocking | nvme_read  |
    };

    topics_in: {
        list: TOPICS_IN_LIST;

        | TopicTy | kind | handler |
        | ------- | ---- | ------- |
    };

    topics_out: {
        list: TOPICS_OUT_LIST;
    };
}

pub fn listen(nvme: Nvme) {
    log::info!("start listening for rpc requests ...");
    let context = Context { nvme };
    let dispatcher = FlashApp::new(context, UsbWireSpawn);
    let vkk = dispatcher.min_key_len();
    let mut server = Server::new(UsbWireTx, UsbWireRx, usb::rx_buf(), dispatcher, vkk);

    loop {
        let err = block_on(server.run());
        log::error!("server run error: {:?}", err);
    }
}

pub struct Context {
    pub nvme: Nvme,
}

impl SpawnContext for Context {
    type SpawnCtxt = ();
    fn spawn_ctxt(&mut self) -> Self::SpawnCtxt {}
}

pub fn block_on<F: Future>(fut: F) -> F::Output {
    fn clone(_: *const ()) -> RawWaker {
        RawWaker::new(core::ptr::null(), &VTABLE)
    }
    fn wake(_: *const ()) {}
    fn wake_by_ref(_: *const ()) {}
    fn drop(_: *const ()) {}
    static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

    let waker = unsafe { Waker::from_raw(RawWaker::new(core::ptr::null(), &VTABLE)) };
    let mut cx = TaskContext::from_waker(&waker);
    let mut fut = core::pin::pin!(fut);
    loop {
        match fut.as_mut().poll(&mut cx) {
            Poll::Ready(v) => return v,
            Poll::Pending => core::hint::spin_loop(),
        }
    }
}
