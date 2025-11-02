use std::{
    error::Error,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, ToSocketAddrs},
};

pub trait Ipv4Source<E: Error> {
    fn get_ipv4(&self) -> Result<impl Iterator<Item = Ipv4Addr>, E>;
}

pub trait Ipv6Source<E: Error> {
    fn get_ipv6(&self) -> Result<impl Iterator<Item = Ipv6Addr>, E>;
}

#[derive(Debug)]
pub struct DomainIpv4Source<'a>(&'a str);

impl<'a> Ipv4Source<std::io::Error> for DomainIpv4Source<'a> {
    fn get_ipv4(&self) -> Result<impl Iterator<Item = Ipv4Addr>, std::io::Error> {
        let socket_addrs = format!("{}:443", self.0).to_socket_addrs()?;
        let ipv4addrs = socket_addrs.filter_map(|socket_addr| match socket_addr.ip() {
            IpAddr::V4(addr) => Some(addr),
            IpAddr::V6(_) => None,
        });

        Ok(ipv4addrs)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, net::Ipv4Addr};

    use crate::ipsource::{DomainIpv4Source, Ipv4Source};

    #[test]
    fn test_domain_ipv4_source() {
        let domain = "dns.quad9.net";
        let source = DomainIpv4Source(domain);
        let ips: HashSet<_> = source.get_ipv4().expect("Failed to resolve {domain}").collect();
        let quadnine = Ipv4Addr::new(9, 9, 9, 9);

        assert!(ips.contains(&quadnine));
    }

    #[test]
    fn test_invalid_domain_ipv4_source() {
        let domain = "qwertzuiopü.local";
        let source = DomainIpv4Source(domain);
        let Err(_error) = source.get_ipv4() else {
            panic!("Successfully resolved {domain}");
        };
    }
}
