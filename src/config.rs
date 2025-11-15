use anyhow::Result;
use base64::{engine::general_purpose::STANDARD as standard_engine, prelude::*};
use serde::Deserialize;
use std::io::{BufReader, Read};

#[derive(Deserialize, Debug)]
pub struct Config {
    pub interface_name: Option<String>,
    pub icanhazip_url: Option<String>,
    pub dyndns: DynDns,
}

#[derive(Deserialize, Debug)]
pub struct DynDns {
    pub domain_name: String,
    pub endpoint: String,
    pub basic_auth: Option<Auth>,
    pub query: Query,
}

#[derive(Deserialize, Debug)]
pub struct Auth {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Debug)]
pub struct Query {
    pub ipv4: String,
    pub ipv6: String,
}

impl Config {
    pub fn try_from_read<R: Read>(reader: R) -> Result<Self> {
        let mut buf_reader = BufReader::new(reader);
        let mut content = String::new();
        let _bytes = buf_reader.read_to_string(&mut content)?;
        let config: Config = toml::from_str(&content)?;

        Ok(config)
    }
}

impl Auth {
    pub fn to_header(&self) -> String {
        let credentials = standard_engine.encode(format!("{}:{}", self.username, self.password));
        format!("Basic {}", credentials)
    }
}
