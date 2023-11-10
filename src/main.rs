mod config;
mod ipaddrs;

use anyhow::{Context, Result};
use config::Config;
use getopts::Options;
use ipaddrs::IpAddrs;
use std::env;
use std::process;
use ureq::Request;

const DEFAULT_CONFIG: &str = "/etc/ipupd/config.toml";

fn main() {
    if let Err(error) = try_main() {
        eprintln!("{}", error);
        process::exit(1);
    }
}

fn try_main() -> Result<()> {
    let mut opts = Options::new();
    opts.optflag("h", "help", "Show help and exit");
    opts.optopt(
        "c",
        "config",
        &format!("Configuration file (default: {})", DEFAULT_CONFIG),
        "FILE",
    );

    let args: Vec<String> = env::args().collect();
    let matches = opts.parse(&args)?;

    if matches.opt_present("h") {
        let usage = format!("Usage: {} [OPTIONS]", args[0]);
        println!("{}", opts.usage(&usage));
        return Ok(());
    }

    let config_file = matches.opt_str("c").unwrap_or(DEFAULT_CONFIG.to_string());
    let config = Config::from_file(&config_file)
        .with_context(|| format!("Could not parse config file {}", &config_file))?;

    let interface = &config.interface;
    let interface_ips = IpAddrs::from_interface(interface);

    if let Some(api) = &config.api {
        // TODO:
        // Get IPv4 from API
        // Adjust interface_ips accordingly
    }

    let domain = &config.domain;
    let domain_ips =
        IpAddrs::from_domain(domain).with_context(|| format!("Could not resolve {}", domain))?;

    if !interface_ips.is_subset(&domain_ips) {
        let request = create_request(&config, &interface_ips);
        let response = request.call().context("Could not perform GET request")?;
        println!(
            "{}",
            response
                .into_string()
                .context("Could not read response body")?
        );
    }

    Ok(())
}

fn create_request(config: &Config, ip_addrs: &IpAddrs) -> Request {
    let ipv4 = ip_addrs
        .iter()
        .find(|ip_addr| ip_addr.is_ipv4())
        .map(|ip_addr| ip_addr.to_string())
        .unwrap_or("0.0.0.0".to_string());

    let ipv6 = ip_addrs
        .iter()
        .find(|ip_addr| ip_addr.is_ipv6())
        .map(|ip_addr| ip_addr.to_string())
        .unwrap_or("::".to_string());

    let request = ureq::get(&config.url)
        .query(&config.query.ipv4, &ipv4)
        .query(&config.query.ipv6, &ipv6);

    if let Some(auth) = &config.basic_auth {
        request.set("Authorization", &auth.to_header())
    } else {
        request
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashSet,
        net::{IpAddr, Ipv4Addr, Ipv6Addr},
    };

    use crate::{
        config::{Config, Query},
        ipaddrs::IpAddrs,
    };

    use super::create_request;

    fn test_request_query(ip_addrs: &IpAddrs, url: &str) {
        let config = Config {
            domain: "foobar.example".to_string(),
            interface: "eth0".to_string(),
            api: None,
            url: "https://dyndns.example".to_string(),
            basic_auth: None,
            query: Query {
                ipv4: "foo".to_string(),
                ipv6: "bar".to_string(),
            },
        };

        let request = create_request(&config, ip_addrs);
        assert_eq!(request.url(), url);
    }

    #[test]
    fn request_query_empty() {
        let ip_addrs = IpAddrs(HashSet::new());
        let url = "https://dyndns.example/?foo=0.0.0.0&bar=%3A%3A";
        test_request_query(&ip_addrs, url);
    }

    #[test]
    fn request_query_v4() {
        let ip_addrs = IpAddrs(HashSet::from([IpAddr::V4(Ipv4Addr::new(192, 0, 2, 0))]));
        let url = "https://dyndns.example/?foo=192.0.2.0&bar=%3A%3A";
        test_request_query(&ip_addrs, url);
    }

    #[test]
    fn request_query_v6() {
        let ip_addrs = IpAddrs(HashSet::from([IpAddr::V6(Ipv6Addr::new(
            0x2001, 0xdb8, 0, 0, 0, 0, 0, 0,
        ))]));
        let url = "https://dyndns.example/?foo=0.0.0.0&bar=2001%3Adb8%3A%3A";
        test_request_query(&ip_addrs, url);
    }

    #[test]
    fn request_query_dual() {
        let ip_addrs = IpAddrs(HashSet::from([
            IpAddr::V4(Ipv4Addr::new(192, 0, 2, 0)),
            IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0)),
        ]));
        let url = "https://dyndns.example/?foo=192.0.2.0&bar=2001%3Adb8%3A%3A";
        test_request_query(&ip_addrs, url);
    }
}
