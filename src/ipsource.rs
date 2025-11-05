use std::{
    cell::RefCell,
    collections::HashSet,
    error::Error,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, ToSocketAddrs},
    vec::IntoIter,
};

pub trait Ipv4Source<E: Error> {
    fn get_ipv4(&self) -> Result<impl Iterator<Item = Ipv4Addr>, E>;
}

pub trait Ipv6Source<E: Error> {
    fn get_ipv6(&self) -> Result<impl Iterator<Item = Ipv6Addr>, E>;
}

#[derive(Debug)]
pub struct DomainIpSource<'a> {
    domain: &'a str,
    socket_addrs: RefCell<HashSet<SocketAddr>>,
}

impl<'a> DomainIpSource<'a> {
    pub fn new(domain: &'a str) -> Self {
        Self {
            domain,
            socket_addrs: RefCell::new(HashSet::new()),
        }
    }

    fn init(&self) -> Result<(), std::io::Error> {
        let socket_addrs = format!("{}:443", self.domain)
            .to_socket_addrs()
            .map(IntoIter::collect)?;
        self.socket_addrs.replace(socket_addrs);

        Ok(())
    }
}

impl<'a> Ipv4Source<std::io::Error> for DomainIpSource<'a> {
    fn get_ipv4(&self) -> Result<impl Iterator<Item = Ipv4Addr>, std::io::Error> {
        if self.socket_addrs.borrow().is_empty() {
            self.init()?;
        }

        let socket_addrs = self.socket_addrs.borrow().clone();
        let ipv4addrs = socket_addrs
            .into_iter()
            .filter_map(|socket_addr| match socket_addr.ip() {
                IpAddr::V4(addr) => Some(addr),
                IpAddr::V6(_) => None,
            });

        Ok(ipv4addrs)
    }
}

impl<'a> Ipv6Source<std::io::Error> for DomainIpSource<'a> {
    fn get_ipv6(&self) -> Result<impl Iterator<Item = Ipv6Addr>, std::io::Error> {
        if self.socket_addrs.borrow().is_empty() {
            self.init()?;
        }

        let socket_addrs = self.socket_addrs.borrow().clone();
        let ipv6addrs = socket_addrs
            .into_iter()
            .filter_map(|socket_addr| match socket_addr.ip() {
                IpAddr::V4(_) => None,
                IpAddr::V6(addr) => Some(addr),
            });

        Ok(ipv6addrs)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, net::{Ipv4Addr, Ipv6Addr}, str::FromStr};

    use crate::ipsource::{DomainIpSource, Ipv4Source, Ipv6Source};

    #[test]
    fn domain_ipv4_source() {
        let domain = "dns.quad9.net";
        let source = DomainIpSource::new(domain);
        let ips: HashSet<_> = source
            .get_ipv4()
            .expect("Failed to resolve {domain}")
            .collect();
        let quadnine = Ipv4Addr::new(9, 9, 9, 9);

        assert!(ips.contains(&quadnine));
    }

    #[test]
    fn invalid_domain_ipv4_source() {
        let domain = "qwertzuiopü.local";
        let source = DomainIpSource::new(domain);
        let Err(_error) = source.get_ipv4() else {
            panic!("Successfully resolved {domain}");
        };
    }

    #[test]
    fn domain_ipv6_source() {
        let domain = "dns.quad9.net";
        let source = DomainIpSource::new(domain);
        let ips: HashSet<_> = source
            .get_ipv6()
            .expect("Failed to resolve {domain}")
            .collect();
        let quadnine = Ipv6Addr::from_str("2620:fe::fe").expect("Failed to create expecte IPv6 address");

        assert!(ips.contains(&quadnine));
    }

    #[test]
    fn invalid_domain_ipv6_source() {
        let domain = "qwertzuiopü.local";
        let source = DomainIpSource::new(domain);
        let Err(_error) = source.get_ipv6() else {
            panic!("Successfully resolved {domain}");
        };
    }

}
