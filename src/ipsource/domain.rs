use std::{
    cell::RefCell,
    collections::HashSet,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, ToSocketAddrs},
    vec::IntoIter,
};

use crate::ipsource::{Ipv4Source, Ipv6Source};

#[derive(Debug)]
pub struct DomainIpSource<'a> {
    domain: &'a str,
    ip_addrs: RefCell<HashSet<IpAddr>>,
}

impl<'a> DomainIpSource<'a> {
    pub fn new(domain: &'a str) -> Self {
        Self {
            domain,
            ip_addrs: RefCell::new(HashSet::new()),
        }
    }

    fn init(&self) -> Result<(), std::io::Error> {
        let socket_addrs = format!("{}:443", self.domain)
            .to_socket_addrs()?;
        let ip_addrs = socket_addrs.map(|socket_addr| socket_addr.ip()).collect();
        self.ip_addrs.replace(ip_addrs);

        Ok(())
    }
}

impl<'a> Ipv4Source<std::io::Error> for DomainIpSource<'a> {
    fn get_ipv4(&self) -> Result<impl Iterator<Item = Ipv4Addr>, std::io::Error> {
        if self.ip_addrs.borrow().is_empty() {
            self.init()?;
        }

        let ip_addrs = self.ip_addrs.borrow().clone();
        let ipv4addrs = ip_addrs
            .into_iter()
            .filter_map(|ip_addr| match ip_addr {
                IpAddr::V4(addr) => Some(addr),
                IpAddr::V6(_) => None,
            });

        Ok(ipv4addrs)
    }
}

impl<'a> Ipv6Source<std::io::Error> for DomainIpSource<'a> {
    fn get_ipv6(&self) -> Result<impl Iterator<Item = Ipv6Addr>, std::io::Error> {
        if self.ip_addrs.borrow().is_empty() {
            self.init()?;
        }

        let ip_addrs = self.ip_addrs.borrow().clone();
        let ipv6addrs = ip_addrs
            .into_iter()
            .filter_map(|ip_addr| match ip_addr {
                IpAddr::V4(_) => None,
                IpAddr::V6(addr) => Some(addr),
            });

        Ok(ipv6addrs)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashSet,
        net::{Ipv4Addr, Ipv6Addr},
    };

    use crate::ipsource::{Ipv4Source, Ipv6Source, domain::DomainIpSource};

    #[test]
    fn domain_ipv4_source() {
        let domain = "localhost";
        let source = DomainIpSource::new(domain);
        let ips: HashSet<_> = source
            .get_ipv4()
            .expect("Failed to resolve {domain}")
            .collect();

        assert!(ips.contains(&Ipv4Addr::LOCALHOST));
    }

    #[test]
    fn invalid_domain_ipv4_source() {
        let domain = "invalidhost";
        let source = DomainIpSource::new(domain);
        let Err(_error) = source.get_ipv4() else {
            panic!("Successfully resolved {domain}");
        };
    }

    #[test]
    fn domain_ipv6_source() {
        let domain = "localhost";
        let source = DomainIpSource::new(domain);
        let ips: HashSet<_> = source
            .get_ipv6()
            .expect("Failed to resolve {domain}")
            .collect();

        assert!(ips.contains(&Ipv6Addr::LOCALHOST));
    }

    #[test]
    fn invalid_domain_ipv6_source() {
        let domain = "invalidhost";
        let source = DomainIpSource::new(domain);
        let Err(_error) = source.get_ipv6() else {
            panic!("Successfully resolved {domain}");
        };
    }
}
