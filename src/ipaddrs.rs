use anyhow::{Context, Result};
use pnet::datalink::interfaces;
use std::{
    collections::HashSet,
    net::{IpAddr, ToSocketAddrs},
    ops::Deref,
};

#[derive(Debug)]
pub struct IpAddrs(pub HashSet<IpAddr>);

impl IpAddrs {
    pub fn from_domain(domain_name: &str) -> Result<Self> {
        let socket_addrs = format!("{domain_name}:443")
            .to_socket_addrs()
            .with_context(|| format!("Could not resolve {domain_name}"))?;

        Ok(Self(
            socket_addrs.map(|socket_addr| socket_addr.ip()).collect(),
        ))
    }

    pub fn from_interface(if_name: &str) -> Self {
        Self(
            interfaces()
                .iter()
                .filter(|interface| interface.name == if_name)
                .flat_map(|interface| interface.ips.clone())
                .map(|network| network.ip())
                .filter(Self::is_global)
                .collect(),
        )
    }

    pub fn from_api(endpoint: &str) -> Result<Self> {
        let result = reqwest::blocking::get(endpoint)
            .with_context(|| format!("Could not GET {endpoint}"))?;
        let content = result.text().context("Could not read response body")?;
        let ip_addr: IpAddr = content
            .trim()
            .parse()
            .context("Could not parse response as IPv4 address")?;
        Ok(Self(HashSet::from([ip_addr])))
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

impl Deref for IpAddrs {
    type Target = HashSet<IpAddr>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
