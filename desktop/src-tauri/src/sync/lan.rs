use std::net::{IpAddr, Ipv4Addr};

/// The private IPv4 address of this machine's LAN interface, if it has one.
pub fn private_ipv4() -> Option<Ipv4Addr> {
    let interfaces = if_addrs::get_if_addrs().ok()?;
    interfaces
        .iter()
        .find_map(|interface| match interface.ip() {
            IpAddr::V4(ip) if is_lan(ip) => Some(ip),
            _ => None,
        })
}

fn is_lan(ip: Ipv4Addr) -> bool {
    ip.is_private() && !ip.is_loopback() && !ip.is_link_local()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_private_ranges_count_as_lan() {
        assert!(is_lan(Ipv4Addr::new(192, 168, 1, 20)));
        assert!(is_lan(Ipv4Addr::new(10, 0, 0, 2)));
        assert!(is_lan(Ipv4Addr::new(172, 20, 1, 1)));
        assert!(!is_lan(Ipv4Addr::new(127, 0, 0, 1)));
        assert!(!is_lan(Ipv4Addr::new(169, 254, 3, 4)));
        assert!(!is_lan(Ipv4Addr::new(8, 8, 8, 8)));
    }
}
