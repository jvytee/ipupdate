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

    fn is_global(ip_addr: &IpAddr) -> bool {
        match ip_addr {
            IpAddr::V6(ipv6) => ipv6
                .segments()
                .first()
                .is_some_and(|segment| 0x0000 < *segment && *segment < 0xf000),
            IpAddr::V4(ipv4) => !(ipv4.is_loopback() || ipv4.is_private() || ipv4.is_link_local()),
        }
    }
}

impl<'a> Ipv4Source<Infallible> for InterfaceIpSource<'a> {
    fn get_ipv4(&self) -> Result<impl Iterator<Item = std::net::Ipv4Addr>, Infallible> {
        if self.ip_addrs.borrow().is_empty() {
            self.init();
        }

        let ip_addrs = self.ip_addrs.borrow().clone();
        let ipv4_addrs = ip_addrs.into_iter().filter_map(|ip_addr| match ip_addr {
            IpAddr::V4(addr) => {
                if Self::is_global(&ip_addr) {
                    Some(addr)
                } else {
                    None
                }
            }
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
            IpAddr::V6(addr) => {
                if Self::is_global(&ip_addr) {
                    Some(addr)
                } else {
                    None
                }
            }
        });

        Ok(ipv6_addrs)
    }
}
