use tracing::{debug, info};
mod ctx;
mod vif;

use crate::ctx::setup_context;
use crate::vif::list_interfaces;
use std::mem::MaybeUninit;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

fn main() {
    let ctx = setup_context().expect("Failed to setup context");

    // for now print received packets
    let mut buf: [MaybeUninit<u8>; 1500] = unsafe { MaybeUninit::uninit().assume_init() };
    loop {
        let n = ctx
            .mroute_socket
            .recv(&mut buf)
            .expect("Failed to receive packet");
        println!("Received {} bytes", n);
    }
}
