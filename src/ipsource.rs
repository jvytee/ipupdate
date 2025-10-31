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
