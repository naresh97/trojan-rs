struct TrojanHandshake {
    password: [u8; 56],
    request: TrojanRequest,
}

struct TrojanRequest {
    command: Command,
    address: Address,
}

struct Endpoint {
    address: Address,
    port: u32,
}

enum Address {
    Ipv6([u8; 6]),
    Ipv4([u8; 4]),
}
