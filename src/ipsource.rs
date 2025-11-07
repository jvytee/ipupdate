mod domain;
mod interface;

use std::{
    error::Error,
    net::{Ipv4Addr, Ipv6Addr},
};

pub trait Ipv4Source<E: Error> {
    fn get_ipv4(&self) -> Result<impl Iterator<Item = Ipv4Addr>, E>;
}

pub trait Ipv6Source<E: Error> {
    fn get_ipv6(&self) -> Result<impl Iterator<Item = Ipv6Addr>, E>;
}
