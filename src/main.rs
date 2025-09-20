use tracing::{debug, info};
mod ctx;
mod igmp;
mod vif;

use crate::ctx::setup_context;
use crate::igmp::decode_igmp;
use crate::vif::list_interfaces;
use socket2::Socket;
use std::io;
use std::mem;
use std::mem::MaybeUninit;
use std::os::unix::io::{AsRawFd, RawFd};
use tokio::io::unix::AsyncFd;
use tokio::sync::mpsc;

use libc::{
    CMSG_DATA, CMSG_FIRSTHDR, CMSG_NXTHDR, IP_PKTINFO, IPPROTO_IP, in_pktinfo, iovec,
    msghdr, recvmsg};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

/// Parse control messages in `msghdr` and extract ifindex via IP_PKTINFO.
unsafe fn parse_cmsgs(msg: &msghdr) -> i32 {
    let mut ifindex: i32 = -1;
    let mut cmsg = CMSG_FIRSTHDR(msg);
    while !cmsg.is_null() {
        if (*cmsg).cmsg_level == IPPROTO_IP && (*cmsg).cmsg_type == IP_PKTINFO {
            let pktinfo: *const in_pktinfo = CMSG_DATA(cmsg) as *const in_pktinfo;
            ifindex = (*pktinfo).ipi_ifindex;
        }
        cmsg = CMSG_NXTHDR(msg, cmsg);
    }
    ifindex
}

/// Receive a packet and extract ifindex using recvmsg + IP_PKTINFO.
fn recv_with_pktinfo(fd: RawFd) -> io::Result<(Vec<u8>, i32)> {
    let mut buf = [0u8; 1500];
    let mut cmsg_space = [0u8; 64]; // ancillary data buffer

    let mut iov = iovec {
        iov_base: buf.as_mut_ptr() as *mut _,
        iov_len: buf.len(),
    };

    let mut msg: msghdr = unsafe { mem::zeroed() };
    msg.msg_iov = &mut iov;
    msg.msg_iovlen = 1;
    msg.msg_control = cmsg_space.as_mut_ptr() as *mut _;
    msg.msg_controllen = cmsg_space.len();

    let n = unsafe { recvmsg(fd, &mut msg, 0) };
    if n < 0 {
        return Err(io::Error::last_os_error());
    }

    let ifindex = unsafe { parse_cmsgs(&msg) };
    Ok((buf[..n as usize].to_vec(), ifindex))
}

/// Packet abstraction
pub struct ReceivedPacket {
    pub data: Vec<u8>,
    pub ifindex: Option<i32>, // None if we didn't use PKTINFO
}

/// Task that uses IP_PKTINFO
pub async fn recv_task_with_pktinfo(
    async_sock: AsyncFd<Socket>,
    tx: mpsc::Sender<ReceivedPacket>,
) -> io::Result<()> {
    loop {
        let mut guard = async_sock.readable().await.unwrap();

        let result = guard.try_io(|inner| {
            let fd = inner.get_ref().as_raw_fd();
            recv_with_pktinfo(fd) // (Vec<u8>, ifindex)
                .map(|(data, ifindex)| ReceivedPacket {
                    data,
                    ifindex: Some(ifindex),
                })
        });

        match result {
            Ok(Ok(pkt)) => {
                println!(
                    "Received {} bytes on ifindex {}",
                    pkt.data.len(),
                    pkt.ifindex.unwrap()
                );
                if tx.send(pkt).await.is_err() {
                    break Ok(());
                }
            }
            Ok(Err(e)) if e.kind() == io::ErrorKind::WouldBlock => continue,
            Ok(Err(e)) => return Err(e),
            Err(_) => continue,
        }
    }
}

/// Task that just uses recv() — no control messages
pub async fn recv_task_plain(
    async_sock: AsyncFd<Socket>,
    tx: mpsc::Sender<ReceivedPacket>,
) -> io::Result<()> {
    loop {
        let mut guard = async_sock.readable().await.unwrap();

        let result = guard.try_io(|inner| {
            let mut buf: [MaybeUninit<u8>; 1500] = unsafe { MaybeUninit::uninit().assume_init() };
            match inner.get_ref().recv(&mut buf) {
                Ok(n) => {
                    let data: Vec<u8> = buf[..n]
                        .iter()
                        .map(|b| unsafe { b.assume_init() })
                        .collect();
                    Ok(ReceivedPacket {
                        data,
                        ifindex: None,
                    })
                }
                Err(e) => Err(e),
            }
        });

        match result {
            Ok(Ok(pkt)) => {
                println!("Received {} bytes", pkt.data.len());
                if tx.send(pkt).await.is_err() {
                    break Ok(());
                }
            }
            Ok(Err(e)) if e.kind() == io::ErrorKind::WouldBlock => continue,
            Ok(Err(e)) => return Err(e),
            Err(_) => continue,
        }
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let ctx = setup_context().expect("Failed to setup context");

    let (tx, mut rx) = mpsc::channel::<ReceivedPacket>(32);
    let async_sock = AsyncFd::new(ctx.mroute_socket).expect("Failed to create async socket");


    let handle = tokio::spawn(recv_task_with_pktinfo(async_sock, tx));

    // let handle = tokio::spawn(recv_task_plain(async_sock, tx));

    tokio::select! {
        _ = handle => {
            info!("Receive task ended");
        }
    }

    Ok(())
}
