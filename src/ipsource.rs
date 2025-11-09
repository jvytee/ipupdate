mod domain;
mod interface;

use std::{
    error::Error,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
};

pub trait Ipv4Source<E: Error> {
    fn get_ipv4(&self) -> Result<impl Iterator<Item = Ipv4Addr>, E>;
}

pub trait Ipv6Source<E: Error> {
    fn get_ipv6(&self) -> Result<impl Iterator<Item = Ipv6Addr>, E>;
}

pub trait MaybeGlobal {
    fn maybe_global(&self) -> bool;
}

impl MaybeGlobal for Ipv4Addr {
    fn maybe_global(&self) -> bool {
        !(self.is_loopback() || self.is_private() || self.is_link_local())
    }
}

impl MaybeGlobal for Ipv6Addr {
    fn maybe_global(&self) -> bool {
        self.segments()
            .first()
            .is_some_and(|segment| 0x0000 < *segment && *segment < 0xf000)
    }
}

impl MaybeGlobal for IpAddr {
    fn maybe_global(&self) -> bool {
        match self {
            IpAddr::V6(ipv6) => ipv6.maybe_global(),
            IpAddr::V4(ipv4) => ipv4.maybe_global(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    use crate::ipsource::MaybeGlobal;

    #[test]
    fn maybe_global() {
        let ipv4 = IpAddr::V4(Ipv4Addr::new(9, 9, 9, 9));
        assert!(ipv4.maybe_global());

        let ipv6 = IpAddr::V6(Ipv6Addr::new(0x2620, 0xfe, 0, 0, 0, 0, 0, 0xfe));
        assert!(ipv6.maybe_global());
    }

    #[test]
    fn may_not_be_global() {
        let ipv4 = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        assert!(!ipv4.maybe_global());

        let ipv6 = IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1));
        assert!(!ipv6.maybe_global());
    }
}
