use std::net::IpAddr;

use ureq::Error;

#[derive(Debug)]
pub struct Client<'a> {
    endpoint: &'a str,
    authorization: Option<&'a str>,
    ipv4_query: &'a str,
    ipv6_query: &'a str,
}

impl<'a> Client<'a> {
    fn sync(&self, ip: &IpAddr) -> Result<(), Error> {
        let agent = ureq::agent();
        let query_key = match ip {
            IpAddr::V4(_) => self.ipv4_query,
            IpAddr::V6(_) => self.ipv6_query,
        };
        let mut request = agent.get(self.endpoint).query(query_key, ip.to_string());

        if let Some(authorization) = self.authorization {
            request = request.header("Authorization", authorization);
        }

        // TODO
        let _response = request.call()?;
        Ok(())
    }
}
