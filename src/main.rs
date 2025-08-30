use tracing::{debug, info};
mod vif;
use std::env;

use crate::vif::list_interfaces;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

fn main() {
    // println!("Hello, world! {}", MRT_INIT);
    // examine_vifs();
    list_interfaces();
}
