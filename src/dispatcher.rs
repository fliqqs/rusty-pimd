use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

/// Packet struct sent from recv task
#[derive(Debug)]
pub struct ReceivedPacket {
    pub ifindex: Option<i32>,
    pub data: Vec<u8>,
}

/// Dispatcher keeps track of sinks/state machines
pub struct Dispatcher {
    sinks: HashMap<i32, mpsc::Sender<ReceivedPacket>>, // keyed by ifindex for now
    rx: mpsc::Receiver<ReceivedPacket>,
}

impl Dispatcher {
    pub fn new(rx: mpsc::Receiver<ReceivedPacket>) -> Self {
        Self {
            sinks: HashMap::new(),
            rx,
        }
    }

    /// Add a sink (e.g. a per-interface IGMP state machine)
    pub fn add_sink(&mut self, ifindex: i32) -> mpsc::Receiver<ReceivedPacket> {
        let (tx, rx) = mpsc::channel(32);
        self.sinks.insert(ifindex, tx);
        rx
    }

    /// Remove a sink when interface disappears
    pub fn remove_sink(&mut self, ifindex: i32) {
        self.sinks.remove(&ifindex);
    }

    /// Core loop – receive packets from socket task and dispatch
    pub async fn run(mut self) {
        info!("Dispatcher started");

        let mut rx = self.rx;
        while let Some(pkt) = rx.recv().await {
            println!(
                "Worker received packet {} with {} bytes",
                pkt.ifindex.unwrap_or(-1),
                pkt.data.len()
            );
        }
        println!("Worker finished");
    }
}
