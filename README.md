# trojan-rs

To circumvent certain firewalls, I previously used [trojan-go](https://github.com/p4gefau1t/trojan-go) which worked pretty well. Unfortunately, the binary size of around 8MB was slightly too large to use on my portable router runnin OpenWRT. So I decided to write an implementation of the Trojan Client/Server in Rust.

The current target is <1MB, which has been achieved.

Based on the Trojan protocol described [here](https://github.com/trojan-gfw/trojan).

## Usage

Modify `client.toml` and `server.toml` appropriately. See [samples](samples/).

### Client

```
trojan-rs client
```

### Server

```
trojan-rs
```

### Logging

Logging can be enabled by setting the `RUSTLOG` environment variable.

```
RUSTLOG=debug trojan-rs
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

## Roadmap

- [ ] add support for the creation of TUN devices