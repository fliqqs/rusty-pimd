use bindgen;
use std::env;
use std::path::PathBuf;

fn main() {
    let bindings = bindgen::Builder::default()
        .header("/usr/include/linux/mroute.h")
        .header("/usr/include/net/if.h")
        .header("/usr/include/linux/sockios.h")
        .blocklist_item("SIOCGIFINDEX")
        .blocklist_item("SIOCGIFFLAGS")
        .blocklist_item("SIOCGIFNETMASK")
        .blocklist_item("SIOCGIFCONF")
        .allowlist_type("vifi_t")
        .allowlist_type("vifctl")
        .allowlist_type("ifreq")
        .allowlist_type("ifconf")
        .allowlist_var("SIOCGIF.*") // ioctl constants
        .allowlist_var("MRT_.*") // multicast routing constants
        .allowlist_var("VIFF_.*")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
