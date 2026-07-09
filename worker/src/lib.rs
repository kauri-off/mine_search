use std::net::Ipv4Addr;

use rand::RngExt;
use rand_chacha::ChaCha8Rng;

pub fn generate_random_ip(rng: &mut ChaCha8Rng) -> Ipv4Addr {
    loop {
        let raw_ip: u32 = rng.random();
        let ip = Ipv4Addr::from(raw_ip);
        let octets = ip.octets();

        if octets[0] == 0 || octets[0] > 223 {
            continue;
        }

        if !is_reserved_ip(octets) {
            return ip;
        }
    }
}

/// Whether a caller-supplied address is safe to probe. Applies the same guards
/// as [`generate_random_ip`] — rejects the 0/>223 first-octet bands and the
/// reserved ranges — so backend-directed targets (update cycle, on-demand
/// ping/scan) cannot aim the worker at internal or bogon hosts. This closes the
/// SSRF-proxy vector where a compromised backend points the worker at private
/// infrastructure. Non-IPv4 inputs (hostnames, IPv6) are rejected.
pub fn is_probeable_ip(ip: &str) -> bool {
    let Ok(addr) = ip.parse::<Ipv4Addr>() else {
        return false;
    };
    let octets = addr.octets();
    if octets[0] == 0 || octets[0] > 223 {
        return false;
    }
    !is_reserved_ip(octets)
}

#[inline(always)]
fn is_reserved_ip(octets: [u8; 4]) -> bool {
    match octets[0] {
        10 => true,                                                    // 10.0.0.0/8
        127 => true,                                                   // 127.0.0.0/8 (loopback)
        172 => octets[1] >= 16 && octets[1] <= 31,                     // 172.16.0.0/12
        192 => octets[1] == 168 || (octets[1] == 0 && octets[2] == 2), // 192.168.0.0/16 + 192.0.2.0/24
        169 => octets[1] == 254, // 169.254.0.0/16 (link-local)
        100 => octets[1] >= 64 && octets[1] <= 127, // 100.64.0.0/10 (CGNAT)
        198 => {
            (octets[1] == 18 || octets[1] == 19)            // 198.18.0.0/15 (benchmarking)
            || (octets[1] == 51 && octets[2] == 100) // 198.51.100.0/24 (documentation)
        }
        203 => octets[1] == 0 && octets[2] == 113, // 203.0.113.0/24 (documentation)
        _ => false,
    }
}
