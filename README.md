# ipupd
Fetches current IP addresses of the machine and compares them with DNS records of a domain.
If there is a difference, records are updated via configurable HTTP request to an API endpoint.

**Work in progress!**
Since current IP addresses are read from the interface directly at the moment, this tool will update IPv6 addresses only in most cases.
If no global address could be detected, it is subsituted by "0.0.0.0" or "::" respectively.

## Installation
Building ipupd requires a recent version of cargo and a rust compiler toolchain.
The recommended way to install those is using [rustup](https://rustup.rs).
Afterwards, a binary can be built from source quite easily:

    git clone https://github.com/jvytee/ipupd.git
    cargo install --path ipupd
    
Alternatively, linux binaries may be downloaded from the [releases section](https://github.com/jvytee/ipupd/releases).

## Usage

    Usage: ipupd [OPTIONS]
    
    Options:
        -h, --help          show help and exit
        -c, --config FILE   configuration file

If no configuration file is given explicitely, ipupd will try to use `config.toml` in the current working directory.

## Configuration
Parameters for ipupd can be specified in a simple [TOML](https://toml.io/en/) file similar to this:

    interface = "wlp4s0"
    domain = "juliantheis.xyz"
    url = "https://dyndns.inwx.com/nic/update"
    
    [basic_auth]
    username = "jvytee"
    password = "blablub"
    
    [query]
    ipv4 = "myip"
    ipv6 = "myipv6"

- `interface` gives the network interface which will be used to estimate current IP addresses
- `domain` is the domain, for which IPs are going to be updated
- `url` specifies an API endpoint by which IPs are updated
- Under `query`, the query parameter names for the API are set. So in combination with `url`, the example will result in a HTTP GET request to `https://dyndns.inwx.com/nic/update?myip=1.2.3.4&myipv6=1234:4567::89`.
- `basic_auth` allows for the configuration of HTTP basic authorization for the API (optional)
