use tracing::{debug, info};
mod ctx;
mod vif;

use crate::ctx::setup_context;
use crate::vif::list_interfaces;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

fn main() {
    let ctx = setup_context().expect("Failed to setup context");

    list_interfaces();
}
