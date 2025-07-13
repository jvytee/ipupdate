mod config;
mod ipaddrs;

use anyhow::{Context, Result};
use config::Config;
use getopts::Options;
use ipaddrs::IpAddrs;
use std::{collections::HashSet, env, net::IpAddr, process};
use ureq::Request;

const DEFAULT_CONFIG: &str = "/etc/ipupdate/config.toml";

fn main() {
    env_logger::init();

    if let Err(error) = try_main() {
        log::error!("{error}");
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
    log::info!("Loading config file {config_file}");
    let config = Config::from_file(&config_file)
        .with_context(|| format!("Could not load config file {config_file}"))?;

    let interface = &config.interface;
    log::info!("Reading current IPv6 addresses of interface {interface}");
    let interface_ips = IpAddrs::from_interface(interface);

    let ip_addrs = match &config.ipv4_url {
        Some(endpoint) => {
            log::info!("Requesting current IPv4 addresses from API {endpoint}");
            let api_ips = IpAddrs::from_api(endpoint)
                .with_context(|| format!("Could not request IPs from {endpoint}"))?;
            IpAddrs(
                interface_ips
                    .union(&api_ips)
                    .copied()
                    .collect::<HashSet<IpAddr>>(),
            )
        }
        None => interface_ips,
    };

    let domain = &config.domain;
    log::info!("Resolving registered IP addresses of domain {domain}");
    let domain_ips =
        IpAddrs::from_domain(domain).with_context(|| format!("Could not resolve {domain}"))?;

    if !ip_addrs.is_subset(&domain_ips) {
        log::info!("Updating IP addresses at {}", &config.dyndns_url);
        let request = create_request(&config, &ip_addrs);
        let response = request
            .call()
            .with_context(|| format!("Could not GET {}", &config.dyndns_url))?;
        let status = response.status();
        let status_text = response.status_text().to_string();
        log::info!("Got {status} {status_text}. Done.");
    } else {
        log::info!("IP adresses are up to date. Done.");
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

    let request = ureq::get(&config.dyndns_url)
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
            ipv4_url: None,
            dyndns_url: "https://dyndns.example".to_string(),
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
