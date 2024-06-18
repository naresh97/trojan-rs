# trojan-rs

![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/naresh97/trojan-rs/rust.yml)

To circumvent certain firewalls, I previously used [trojan-go](https://github.com/p4gefau1t/trojan-go) which worked pretty well. Unfortunately, the binary size of around 8MB was slightly too large to use on my portable router runnin OpenWRT. So I decided to write an implementation of the Trojan Client/Server in Rust.

The goal is to keep the binary under 1MB.

Based on the Trojan protocol described [here](https://github.com/trojan-gfw/trojan).

## Usage

Modify `client.toml` and `server.toml` appropriately. See [samples](samples/).

```
trojan-rs [OPTIONS] <COMMAND> [ADAPTER]

Commands:
  server  runs the Trojan Server
  client  runs the Trojan Client with the specified adapter

Options:
  --config <FILE>   defaults to "./server.toml" or "./client.toml"
  --log             defaults to INFO
                    values: DEBUG, INFO, WARN, ERROR

Adapter:
  --adapter <ADAPTER_TYPE>  used in client mode
                            defaults to socks5
                            values: socks5 tun
```

## Architecture

On the client side:

```
CLIENT <-> SOCKS5 SERVER <-> TROJAN CLIENT <-> INTERNET
```

On the server side:
```
INTERNET <-> TROJAN SERVER  <-> SERVER
                            <-> FALLBACK
```
