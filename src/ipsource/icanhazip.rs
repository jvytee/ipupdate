use std::{
    fmt::{self, Display, Formatter},
    net::{AddrParseError, Ipv4Addr, Ipv6Addr},
};

use reqwest::{self, blocking};
use url::Url;

use crate::ipsource::{Ipv4Source, Ipv6Source};

#[derive(Debug)]
pub struct ICanHazIpSource<'a>(&'a Url);

impl<'a> Ipv4Source<Error> for ICanHazIpSource<'a> {
    fn get_ipv4(&self) -> Result<impl Iterator<Item = Ipv4Addr>, Error> {
        let response = blocking::get(self.0.as_str())?;
        let content = response.text()?;
        let addr: Ipv4Addr = content.trim().parse()?;

        Ok([addr].into_iter())
    }
}

impl<'a> Ipv6Source<Error> for ICanHazIpSource<'a> {
    fn get_ipv6(&self) -> Result<impl Iterator<Item = Ipv6Addr>, Error> {
        let response = blocking::get(self.0.as_str())?;
        let content = response.text()?;
        let addr: Ipv6Addr = content.trim().parse()?;

        Ok([addr].into_iter())
    }
}

#[derive(Debug)]
pub enum Error {
    RequestError(reqwest::Error),
    ResponseError(AddrParseError),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::RequestError(err) => write!(f, "Failed to request IP address: {err}"),
            Self::ResponseError(err) => {
                write!(f, "Failed to parse IP address from response: {err}")
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self::RequestError(value)
    }
}

impl From<AddrParseError> for Error {
    fn from(value: AddrParseError) -> Self {
        Self::ResponseError(value)
    }
}

#[cfg(test)]
mod tests {
    // TODO Inject HTTP client via trait, write unit tests
}
