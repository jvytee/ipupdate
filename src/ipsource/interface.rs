use std::{cell::RefCell, collections::HashSet, convert::Infallible, net::IpAddr};

use pnet::datalink;

use crate::ipsource::{Ipv4Source, Ipv6Source};

pub struct InterfaceIpSource<'a> {
    name: &'a str,
    ip_addrs: RefCell<HashSet<IpAddr>>,
}

impl<'a> InterfaceIpSource<'a> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            ip_addrs: RefCell::new(HashSet::new()),
        }
    }

    fn init(&self) {
        let ip_addrs = datalink::interfaces()
            .iter()
            .filter_map(|interface| {
                if interface.name == self.name {
                    let ip_addrs: Vec<_> = interface
                        .ips
                        .iter()
                        .map(|ip_network| ip_network.ip())
                        .collect();
                    Some(ip_addrs)
                } else {
                    None
                }
            })
            .flatten()
            .collect();
        self.ip_addrs.replace(ip_addrs);
    }
}

impl<'a> Ipv4Source<Infallible> for InterfaceIpSource<'a> {
    fn get_ipv4(&self) -> Result<impl Iterator<Item = std::net::Ipv4Addr>, Infallible> {
        if self.ip_addrs.borrow().is_empty() {
            self.init();
        }

        let ip_addrs = self.ip_addrs.borrow().clone();
        let ipv4_addrs = ip_addrs.into_iter().filter_map(|ip_addr| match ip_addr {
            IpAddr::V4(addr) => Some(addr),
            IpAddr::V6(_) => None,
        });

        Ok(ipv4_addrs)
    }
}

impl<'a> Ipv6Source<Infallible> for InterfaceIpSource<'a> {
    fn get_ipv6(&self) -> Result<impl Iterator<Item = std::net::Ipv6Addr>, Infallible> {
        if self.ip_addrs.borrow().is_empty() {
            self.init();
        }

        let ip_addrs = self.ip_addrs.borrow().clone();
        let ipv6_addrs = ip_addrs.into_iter().filter_map(|ip_addr| match ip_addr {
            IpAddr::V4(_) => None,
            IpAddr::V6(addr) => Some(addr),
        });

        Ok(ipv6_addrs)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashSet,
        net::{Ipv4Addr, Ipv6Addr},
    };

    use pnet::datalink;

    use crate::ipsource::{Ipv4Source, Ipv6Source, interface::InterfaceIpSource};

    fn get_loopback_name() -> Option<String> {
        datalink::interfaces()
            .into_iter()
            .filter_map(|interface| {
                if interface.is_loopback() {
                    Some(interface.name)
                } else {
                    None
                }
            })
            .next()
    }

    #[test]
    fn interface_ipv4_source() {
        let loopback = get_loopback_name().expect("No loopback interface found");
        let source = InterfaceIpSource::new(&loopback);
        let ips: HashSet<_> = source
            .get_ipv4()
            .expect("Infallible function failed")
            .collect();

        assert!(ips.contains(&Ipv4Addr::LOCALHOST));
    }

    #[test]
    fn interface_ipv6_source() {
        let loopback = get_loopback_name().expect("No loopback interface found");
        let source = InterfaceIpSource::new(&loopback);
        let ips: HashSet<_> = source
            .get_ipv6()
            .expect("Infallible function failed")
            .collect();

        assert!(ips.contains(&Ipv6Addr::LOCALHOST));
    }

    #[test]
    fn invalid_interface_ipv4_source() {
        let random = "foobar1337";
        let source = InterfaceIpSource::new(random);
        let ips: HashSet<_> = source
            .get_ipv4()
            .expect("Infallible function failed")
            .collect();

        assert!(ips.is_empty());
    }

    #[test]
    fn invalid_interface_ipv6_source() {
        let random = "foobar1337";
        let source = InterfaceIpSource::new(random);
        let ips: HashSet<_> = source
            .get_ipv6()
            .expect("Infallible function failed")
            .collect();

        assert!(ips.is_empty());
    }
}
