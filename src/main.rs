mod config;
mod ipaddrs;
mod ipsource;

use anyhow::{Context, Error, Result};
use config::Config;
use getopts::Options;
use ipaddrs::IpAddrs;
use reqwest::blocking::{Client, Request};
use std::{collections::HashSet, env, fs, net::IpAddr, process};

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
        &format!("Configuration file (default: {DEFAULT_CONFIG})"),
        "FILE",
    );

    let args: Vec<String> = env::args().collect();
    let matches = opts.parse(&args)?;

    if matches.opt_present("h") {
        let usage = format!("Usage: {} [OPTIONS]", args[0]);
        println!("{}", opts.usage(&usage));
        return Ok(());
    }

    let filename = matches.opt_str("c").unwrap_or(DEFAULT_CONFIG.to_string());
    log::info!("Loading config file {filename}");
    let config_file = fs::File::open(&filename)?;
    let config = Config::try_from_read(&config_file)
        .with_context(|| format!("Could not load config file {filename}"))?;

    let Some(interface) = &config.interface_name else {
        return Err(Error::msg("No interface name specified!"));
    };
    log::info!("Reading current IPv6 addresses of interface {interface}");
    let interface_ips = IpAddrs::from_interface(interface);

    let client = Client::new();
    let ip_addrs = match &config.icanhazip_url {
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

    let domain = &config.dyndns.domain_name;
    log::info!("Resolving registered IP addresses of domain {domain}");
    let domain_ips =
        IpAddrs::from_domain(domain).with_context(|| format!("Could not resolve {domain}"))?;

    if !ip_addrs.is_subset(&domain_ips) {
        log::info!("Updating IP addresses at {}", &config.dyndns.endpoint);

        let request =
            create_request(&client, &config, &ip_addrs).context("Could not create request")?;
        let response = client
            .execute(request)
            .with_context(|| format!("Could not GET {}", &config.dyndns.endpoint))?;
        let status = response.status();
        log::info!("Got status code {status}. Done.");
    } else {
        log::info!("IP adresses are up to date. Done.");
    }

    Ok(())
}

fn create_request(
    client: &Client,
    config: &Config,
    ip_addrs: &IpAddrs,
) -> Result<Request, reqwest::Error> {
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

    let mut request_builder = client.get(&config.dyndns.endpoint).query(&[
        (&config.dyndns.query.ipv4, &ipv4),
        (&config.dyndns.query.ipv6, &ipv6),
    ]);

    if let Some(auth) = &config.dyndns.basic_auth {
        request_builder = request_builder.header("Authorization", &auth.to_header());
    }

    request_builder.build()
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashSet,
        net::{IpAddr, Ipv4Addr, Ipv6Addr},
    };

    use reqwest::blocking::Client;

    use crate::{
        config::{Config, DynDns, Query},
        ipaddrs::IpAddrs,
    };

    use super::create_request;

    fn test_request_query(ip_addrs: &IpAddrs, url: &str) {
        let config = Config {
            interface_name: Some("eth0".to_string()),
            icanhazip_url: None,
            dyndns: DynDns {
                domain_name: "foobar.example".to_string(),
                endpoint: "https://dyndns.example/".to_string(),
                basic_auth: None,
                query: Query {
                    ipv4: "foo".to_string(),
                    ipv6: "bar".to_string(),
                },
            },
        };

        let client = Client::new();
        let request = create_request(&client, &config, ip_addrs).expect("Could not create request");

        assert_eq!(request.url().as_str(), url);
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
