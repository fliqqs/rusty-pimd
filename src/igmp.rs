include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct IpHdr {
    pub ihl_version: u8, // Version + IHL
    pub tos: u8,         // Type of service
    pub tot_len: u16,    // Total length
    pub id: u16,         // Identification
    pub frag_off: u16,   // Fragment offset field
    pub ttl: u8,         // Time to live
    pub protocol: u8,    // Protocol
    pub check: u16,      // Checksum
    pub saddr: u32,      // Source address
    pub daddr: u32,      // Destination address
                         // Options may follow (if ihl > 5)
}

// take a buffer and return an IpHdr
impl IpHdr {
    pub fn decode(&mut self, buf: Vec<u8>) -> () {
        if buf.len() < 20 {
            eprintln!("Buffer too small for IP header");
            return;
        }

        self.ihl_version = buf[0];
        self.tos = buf[1];
        self.tot_len = u16::from_be_bytes([buf[2], buf[3]]);
        self.id = u16::from_be_bytes([buf[4], buf[5]]);
        self.frag_off = u16::from_be_bytes([buf[6], buf[7]]);
        self.ttl = buf[8];
        self.protocol = buf[9];
        self.check = u16::from_be_bytes([buf[10], buf[11]]);
        self.saddr = u32::from_be_bytes([buf[12], buf[13], buf[14], buf[15]]);
        self.daddr = u32::from_be_bytes([buf[16], buf[17], buf[18], buf[19]]);
    }
    pub fn pretty_print(&self) {
        let saddr = std::net::Ipv4Addr::from(self.saddr);
        let daddr = std::net::Ipv4Addr::from(self.daddr);
        println!("IP Header:");
        println!("  Version: {}", self.ihl_version >> 4);
        println!("  IHL: {}", self.ihl_version & 0x0F);
        println!("  TOS: {}", self.tos);
        println!("  Total Length: {}", self.tot_len);
        println!("  ID: {}", self.id);
        println!("  Fragment Offset: {}", self.frag_off);
        println!("  TTL: {}", self.ttl);
        println!("  Protocol: {}", self.protocol);
        println!("  Checksum: {}", self.check);
        println!("  Source Address: {}", saddr);
        println!("  Destination Address: {}", daddr);
    }
}

pub fn decode_igmp(packet: Vec<u8>) {
    let mut ip_hdr: IpHdr = unsafe { std::ptr::read(packet.as_ptr() as *const _) };
    ip_hdr.decode(packet);
    ip_hdr.pretty_print();
}
