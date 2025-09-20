use tracing::{debug, info};
mod ctx;
mod igmp;
mod vif;

use crate::ctx::setup_context;
use crate::igmp::decode_igmp;
use crate::vif::list_interfaces;
use std::mem::MaybeUninit;
use tokio::io::unix::AsyncFd;
use tokio::sync::mpsc;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let ctx = setup_context().expect("Failed to setup context");

    let (tx, mut rx) = mpsc::channel::<Vec<u8>>(32);

    let async_sock = AsyncFd::new(ctx.mroute_socket).expect("Failed to create async socket");

    // recv task
    let recv_task = tokio::spawn(async move {
        loop {
            let mut guard = async_sock.readable().await.unwrap();
            let result = guard.try_io(|inner| {
                // allocate MaybeUninit buffer each recv
                let mut buf: [MaybeUninit<u8>; 1500] =
                    unsafe { MaybeUninit::uninit().assume_init() };

                match inner.get_ref().recv(&mut buf) {
                    Ok(n) => {
                        println!("Received {} bytes", n);
                        let packet: Vec<u8> = buf[..n]
                            .iter()
                            .map(|b| unsafe { b.assume_init() })
                            .collect();
                        Ok(packet)
                    }
                    Err(e) => Err(e),
                }
            });

            match result {
                Ok(Ok(packet)) => {
                    debug!("Received {} bytes", packet.len());

                    // print buffer
                    for byte in &packet {
                        print!("{:02x} ", byte);
                    }
                    println!();

                    decode_igmp(packet.clone());

                    if let Err(e) = tx.send(packet).await {
                        eprintln!("Receiver dropped: {}", e);
                        break;
                    }
                }
                Ok(Err(e)) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // didn't consume readiness, loop will retry
                    continue;
                }
                Ok(Err(e)) => {
                    eprintln!("recv error: {}", e);
                    break;
                }
                Err(_would_block) => continue, // Tokio says: not ready yet
            }
        }
    });

    tokio::select! {
        _ = recv_task => {
            info!("Receive task ended");
        }
        // Add other tasks or signals to listen for here
    }

    Ok(())
}
