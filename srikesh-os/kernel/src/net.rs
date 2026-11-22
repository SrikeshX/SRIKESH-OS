//! SRIKESH OS - Real Bare-Metal Network Protocol Stack
//!
//! Implements genuine wire-level networking over Intel PRO/1000 (e1000):
//! - Ethernet II Framing (ARP 0x0806, IPv4 0x0800)
//! - ARP Request / Reply Resolution
//! - DHCP Client (Discover -> Offer -> Request -> Ack) for real IP/Gateway/DNS
//! - ICMP Echo Request (Ping)
//! - UDP & RFC 1035 DNS A-Record Resolver
//! - TCP 3-Way Handshake + Live HTTP/1.0 GET Client

use crate::serial;
use crate::e1000::{self, E1000};

pub const ETHERTYPE_IPV4: u16 = 0x0800;
pub const ETHERTYPE_ARP: u16 = 0x0806;

pub const IP_PROTO_ICMP: u8 = 1;
pub const IP_PROTO_TCP: u8 = 6;
pub const IP_PROTO_UDP: u8 = 17;

/// RFC 1071 one's complement Internet Checksum
pub fn internet_checksum(data: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    let mut i = 0;
    while i + 1 < data.len() {
        let word = ((data[i] as u32) << 8) | (data[i + 1] as u32);
        sum = sum.wrapping_add(word);
        i += 2;
    }
    if i < data.len() {
        sum = sum.wrapping_add((data[i] as u32) << 8);
    }
    while (sum >> 16) != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    !(sum as u16)
}

fn transport_checksum(src_ip: [u8; 4], dst_ip: [u8; 4], proto: u8, payload: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    sum += ((src_ip[0] as u32) << 8) | (src_ip[1] as u32);
    sum += ((src_ip[2] as u32) << 8) | (src_ip[3] as u32);
    sum += ((dst_ip[0] as u32) << 8) | (dst_ip[1] as u32);
    sum += ((dst_ip[2] as u32) << 8) | (dst_ip[3] as u32);
    sum += proto as u32;
    sum += payload.len() as u32;

    let mut i = 0;
    while i + 1 < payload.len() {
        sum += ((payload[i] as u32) << 8) | (payload[i + 1] as u32);
        i += 2;
    }
    if i < payload.len() {
        sum += (payload[i] as u32) << 8;
    }
    while (sum >> 16) != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    !(sum as u16)
}

fn build_eth_ipv4_header(
    pkt: &mut [u8],
    dst_mac: [u8; 6],
    src_mac: [u8; 6],
    src_ip: [u8; 4],
    dst_ip: [u8; 4],
    proto: u8,
    payload_len: usize,
) {
    pkt[0..6].copy_from_slice(&dst_mac);
    pkt[6..12].copy_from_slice(&src_mac);
    pkt[12] = 0x08;
    pkt[13] = 0x00;

    let total_len = (20 + payload_len) as u16;
    pkt[14] = 0x45;
    pkt[15] = 0x00;
    pkt[16] = (total_len >> 8) as u8;
    pkt[17] = (total_len & 0xFF) as u8;
    pkt[18] = 0x12;
    pkt[19] = 0x34;
    pkt[20] = 0x40;
    pkt[21] = 0x00;
    pkt[22] = 64;
    pkt[23] = proto;
    pkt[24] = 0;
    pkt[25] = 0;
    pkt[26..30].copy_from_slice(&src_ip);
    pkt[30..34].copy_from_slice(&dst_ip);
    let csum = internet_checksum(&pkt[14..34]);
    pkt[24] = (csum >> 8) as u8;
    pkt[25] = (csum & 0xFF) as u8;
}

/// Reply to an incoming ARP request if it asks for our IP
fn maybe_reply_arp(rx: &[u8], my_mac: [u8; 6], my_ip: [u8; 4]) {
    if rx.len() < 42 || rx[12] != 0x08 || rx[13] != 0x06 {
        return;
    }
    let op = ((rx[20] as u16) << 8) | (rx[21] as u16);
    if op == 1 && rx[38..42] == my_ip && my_ip != [0, 0, 0, 0] {
        let mut rep = [0u8; 42];
        rep[0..6].copy_from_slice(&rx[22..28]);
        rep[6..12].copy_from_slice(&my_mac);
        rep[12] = 0x08;
        rep[13] = 0x06;
        rep[14..20].copy_from_slice(&[0, 1, 8, 0, 6, 4]);
        rep[20] = 0;
        rep[21] = 2; // Reply
        rep[22..28].copy_from_slice(&my_mac);
        rep[28..32].copy_from_slice(&my_ip);
        rep[32..38].copy_from_slice(&rx[22..28]);
        rep[38..42].copy_from_slice(&rx[28..32]);
        let _ = e1000::send_packet(&rep);
    }
}

/// Real ARP resolution for target IPv4 address
pub fn resolve_arp(target_ip: [u8; 4]) -> Option<[u8; 6]> {
    let (my_mac, my_ip) = {
        let nic = E1000.lock();
        if !nic.initialized {
            return None;
        }
        (nic.mac_addr, nic.ip_addr)
    };

    let mut req = [0u8; 42];
    req[0..6].copy_from_slice(&[0xFF; 6]);
    req[6..12].copy_from_slice(&my_mac);
    req[12] = 0x08;
    req[13] = 0x06;
    req[14..20].copy_from_slice(&[0, 1, 8, 0, 6, 4]);
    req[20] = 0;
    req[21] = 1; // Request
    req[22..28].copy_from_slice(&my_mac);
    req[28..32].copy_from_slice(&my_ip);
    req[32..38].copy_from_slice(&[0; 6]);
    req[38..42].copy_from_slice(&target_ip);

    let mut rx = [0u8; 1600];
    for _attempt in 0..3 {
        let _ = e1000::send_packet(&req);
        for _ in 0..150_000 {
            if let Some(len) = e1000::recv_packet(&mut rx) {
                if len >= 42 && rx[12] == 0x08 && rx[13] == 0x06 {
                    let op = ((rx[20] as u16) << 8) | (rx[21] as u16);
                    if op == 2 && rx[28..32] == target_ip {
                        let mut mac = [0u8; 6];
                        mac.copy_from_slice(&rx[22..28]);
                        return Some(mac);
                    }
                    maybe_reply_arp(&rx[..len], my_mac, my_ip);
                }
            }
            core::hint::spin_loop();
        }
    }
    None
}

/// Real DHCP Client (Discover -> Offer -> Request -> Ack)
pub fn run_dhcp() -> bool {
    let my_mac = {
        let nic = E1000.lock();
        if !nic.initialized {
            return false;
        }
        nic.mac_addr
    };

    serial::write_str("[DHCP] Sending DHCP Discover on e1000...\n");
    let xid: [u8; 4] = [0x53, 0x52, 0x4B, 0x01];

    let send_dhcp_msg = |msg_type: u8, req_ip: Option<[u8; 4]>, srv_ip: Option<[u8; 4]>| {
        let mut pkt = [0u8; 342]; // 14 Eth + 20 IP + 8 UDP + 300 BOOTP/DHCP
        let udp_off = 34;
        let dhcp_off = 42;

        // BOOTP Header
        pkt[dhcp_off] = 1; // BOOTREQUEST
        pkt[dhcp_off + 1] = 1; // Ethernet
        pkt[dhcp_off + 2] = 6; // MAC len
        pkt[dhcp_off + 4..dhcp_off + 8].copy_from_slice(&xid);
        pkt[dhcp_off + 10] = 0x80; // Broadcast flag
        pkt[dhcp_off + 28..dhcp_off + 34].copy_from_slice(&my_mac);
        // DHCP Magic Cookie
        pkt[dhcp_off + 236..dhcp_off + 240].copy_from_slice(&[99, 130, 83, 99]);

        let mut opt = dhcp_off + 240;
        // Option 53: DHCP Message Type
        pkt[opt] = 53; pkt[opt + 1] = 1; pkt[opt + 2] = msg_type; opt += 3;
        if let Some(ip) = req_ip {
            pkt[opt] = 50; pkt[opt + 1] = 4; pkt[opt + 2..opt + 6].copy_from_slice(&ip); opt += 6;
        }
        if let Some(sip) = srv_ip {
            pkt[opt] = 54; pkt[opt + 1] = 4; pkt[opt + 2..opt + 6].copy_from_slice(&sip); opt += 6;
        }
        // Option 55: Parameter Request List (Subnet, Router, DNS)
        pkt[opt] = 55; pkt[opt + 1] = 3; pkt[opt + 2] = 1; pkt[opt + 3] = 3; pkt[opt + 4] = 6; opt += 5;
        pkt[opt] = 255; // End

        // UDP header (68 -> 67)
        let udp_len: u16 = 8 + 300;
        pkt[udp_off] = 0; pkt[udp_off + 1] = 68;
        pkt[udp_off + 2] = 0; pkt[udp_off + 3] = 67;
        pkt[udp_off + 4] = (udp_len >> 8) as u8;
        pkt[udp_off + 5] = (udp_len & 0xFF) as u8;
        pkt[udp_off + 6] = 0; pkt[udp_off + 7] = 0;

        build_eth_ipv4_header(&mut pkt, [0xFF; 6], my_mac, [0, 0, 0, 0], [255, 255, 255, 255], IP_PROTO_UDP, udp_len as usize);
        let _ = e1000::send_packet(&pkt);
    };

    let mut rx = [0u8; 1600];
    let mut offered_ip: Option<[u8; 4]> = None;
    let mut server_ip: [u8; 4] = [10, 0, 2, 2];
    let mut gateway_ip: [u8; 4] = [10, 0, 2, 2];
    let mut dns_ip: [u8; 4] = [10, 0, 2, 3];

    // Step 1: Discover -> Offer
    for _ in 0..2 {
        send_dhcp_msg(1, None, None);
        for _ in 0..400_000 {
            if let Some(len) = e1000::recv_packet(&mut rx) {
                if len >= 285 && rx[12] == 0x08 && rx[13] == 0x00 && rx[23] == IP_PROTO_UDP {
                    let dst_port = ((rx[36] as u16) << 8) | (rx[37] as u16);
                    if dst_port == 68 && rx[46..50] == xid {
                        let mut yi = [0u8; 4];
                        yi.copy_from_slice(&rx[58..62]);
                        // Parse options
                        let mut p = 42 + 240;
                        let mut is_offer = false;
                        while p + 2 < len && rx[p] != 255 {
                            let code = rx[p];
                            let olen = rx[p + 1] as usize;
                            if p + 2 + olen > len { break; }
                            if code == 53 && olen == 1 && rx[p + 2] == 2 { is_offer = true; }
                            if code == 54 && olen == 4 { server_ip.copy_from_slice(&rx[p + 2..p + 6]); }
                            if code == 3 && olen >= 4 { gateway_ip.copy_from_slice(&rx[p + 2..p + 6]); }
                            if code == 6 && olen >= 4 { dns_ip.copy_from_slice(&rx[p + 2..p + 6]); }
                            p += 2 + olen;
                        }
                        if is_offer && yi != [0, 0, 0, 0] {
                            offered_ip = Some(yi);
                            break;
                        }
                    }
                }
            }
            core::hint::spin_loop();
        }
        if offered_ip.is_some() { break; }
    }

    let assigned_ip = match offered_ip {
        Some(ip) => ip,
        None => {
            serial::write_str("[DHCP] No DHCP Offer received.\n");
            return false;
        }
    };

    // Step 2: Request -> Ack
    send_dhcp_msg(3, Some(assigned_ip), Some(server_ip));
    for _ in 0..400_000 {
        if let Some(len) = e1000::recv_packet(&mut rx) {
            if len >= 285 && rx[12] == 0x08 && rx[13] == 0x00 && rx[23] == IP_PROTO_UDP {
                let dst_port = ((rx[36] as u16) << 8) | (rx[37] as u16);
                if dst_port == 68 && rx[46..50] == xid {
                    let mut p = 42 + 240;
                    while p + 2 < len && rx[p] != 255 {
                        let code = rx[p];
                        let olen = rx[p + 1] as usize;
                        if p + 2 + olen > len { break; }
                        if code == 53 && olen == 1 && rx[p + 2] == 5 {
                            let mut nic = E1000.lock();
                            nic.ip_addr = assigned_ip;
                            nic.gateway_ip = gateway_ip;
                            nic.dns_ip = dns_ip;
                            nic.dhcp_ok = true;
                            serial::write_str("[DHCP] Lease ACK received! Real IP assigned.\n");
                            return true;
                        }
                        p += 2 + olen;
                    }
                }
            }
        }
        core::hint::spin_loop();
    }
    false
}

/// Real RFC 1035 UDP DNS A-record resolution
pub fn resolve_dns(hostname: &str) -> Option<[u8; 4]> {
    let (my_mac, my_ip, gw_ip, dns_ip) = {
        let nic = E1000.lock();
        if !nic.initialized || nic.ip_addr == [0, 0, 0, 0] {
            return None;
        }
        (nic.mac_addr, nic.ip_addr, nic.gateway_ip, nic.dns_ip)
    };

    let gw_mac = resolve_arp(gw_ip)?;
    let mut pkt = [0u8; 512];
    let udp_off = 34;
    let dns_off = 42;

    // DNS Header (12 bytes)
    pkt[dns_off] = 0x53; pkt[dns_off + 1] = 0x52; // ID
    pkt[dns_off + 2] = 0x01; pkt[dns_off + 3] = 0x00; // Standard Query, RD=1
    pkt[dns_off + 4] = 0x00; pkt[dns_off + 5] = 0x01; // QDCOUNT = 1

    let mut pos = dns_off + 12;
    for label in hostname.split('.') {
        let lb = label.as_bytes();
        if lb.is_empty() || pos + 1 + lb.len() >= 480 { continue; }
        pkt[pos] = lb.len() as u8;
        pos += 1;
        pkt[pos..pos + lb.len()].copy_from_slice(lb);
        pos += lb.len();
    }
    pkt[pos] = 0; pos += 1; // Root terminator
    pkt[pos] = 0; pkt[pos + 1] = 1; pos += 2; // QTYPE = A
    pkt[pos] = 0; pkt[pos + 1] = 1; pos += 2; // QCLASS = IN

    let dns_len = pos - dns_off;
    let udp_len = (8 + dns_len) as u16;
    let src_port: u16 = 53053;
    pkt[udp_off] = (src_port >> 8) as u8; pkt[udp_off + 1] = (src_port & 0xFF) as u8;
    pkt[udp_off + 2] = 0; pkt[udp_off + 3] = 53;
    pkt[udp_off + 4] = (udp_len >> 8) as u8; pkt[udp_off + 5] = (udp_len & 0xFF) as u8;
    pkt[udp_off + 6] = 0; pkt[udp_off + 7] = 0;

    build_eth_ipv4_header(&mut pkt, gw_mac, my_mac, my_ip, dns_ip, IP_PROTO_UDP, udp_len as usize);

    let mut rx = [0u8; 1600];
    for _ in 0..2 {
        let _ = e1000::send_packet(&pkt[..pos]);
        for _ in 0..400_000 {
            if let Some(len) = e1000::recv_packet(&mut rx) {
                maybe_reply_arp(&rx[..len], my_mac, my_ip);
                if len >= 54 && rx[12] == 0x08 && rx[13] == 0x00 && rx[23] == IP_PROTO_UDP {
                    let dst_port = ((rx[36] as u16) << 8) | (rx[37] as u16);
                    if dst_port == src_port && rx[42] == 0x53 && rx[43] == 0x52 {
                        let ancount = ((rx[48] as u16) << 8) | (rx[49] as u16);
                        if ancount == 0 { return None; }
                        // Skip question section
                        let mut p = 42 + 12;
                        while p < len && rx[p] != 0 {
                            if (rx[p] & 0xC0) == 0xC0 { p += 2; break; }
                            p += 1 + rx[p] as usize;
                        }
                        if p < len && rx[p] == 0 { p += 1; }
                        p += 4; // QTYPE + QCLASS

                        // Parse answer records
                        for _ in 0..ancount {
                            if p + 12 > len { break; }
                            if (rx[p] & 0xC0) == 0xC0 {
                                p += 2;
                            } else {
                                while p < len && rx[p] != 0 { p += 1 + rx[p] as usize; }
                                p += 1;
                            }
                            if p + 10 > len { break; }
                            let rtype = ((rx[p] as u16) << 8) | (rx[p + 1] as u16);
                            let rdlen = ((rx[p + 8] as u16) << 8) | (rx[p + 9] as u16);
                            p += 10;
                            if rtype == 1 && rdlen == 4 && p + 4 <= len {
                                return Some([rx[p], rx[p + 1], rx[p + 2], rx[p + 3]]);
                            }
                            p += rdlen as usize;
                        }
                    }
                }
            }
            core::hint::spin_loop();
        }
    }
    None
}

/// Real TCP 3-Way Handshake + Live HTTP/1.0 GET Client over Intel e1000
pub fn http_get(host: &str, path: &str, out: &mut [u8]) -> Result<(usize, [u8; 4]), &'static str> {
    // Auto-run DHCP if link is up but lease not yet acquired
    if e1000::is_link_up() && E1000.lock().ip_addr == [0, 0, 0, 0] {
        let _ = run_dhcp();
    }
    let (my_mac, my_ip, gw_ip) = {
        let nic = E1000.lock();
        if !nic.initialized {
            return Err("No active Ethernet hardware (e1000)");
        }
        if nic.ip_addr == [0, 0, 0, 0] {
            return Err("No DHCP IP lease — network offline");
        }
        (nic.mac_addr, nic.ip_addr, nic.gateway_ip)
    };

    let dst_ip = resolve_dns(host).ok_or("DNS resolution failed (check internet/NAT)")?;
    let gw_mac = resolve_arp(gw_ip).ok_or("Gateway ARP resolution failed")?;

    let ticks = crate::interrupts::TIMER_TICKS.load(core::sync::atomic::Ordering::Relaxed);
    let src_port: u16 = 49152 + ((ticks % 10000) as u16);
    let dst_port: u16 = 80;
    let mut my_seq: u32 = 0x1000_2000 + (ticks as u32);
    let mut peer_ack: u32 = 0;

    let send_tcp = |flags: u8, seq: u32, ack: u32, data: &[u8]| {
        let mut pkt = [0u8; 600];
        let tcp_off = 34;
        let tcp_len = 20 + data.len();
        pkt[tcp_off] = (src_port >> 8) as u8;
        pkt[tcp_off + 1] = (src_port & 0xFF) as u8;
        pkt[tcp_off + 2] = (dst_port >> 8) as u8;
        pkt[tcp_off + 3] = (dst_port & 0xFF) as u8;
        pkt[tcp_off + 4..tcp_off + 8].copy_from_slice(&seq.to_be_bytes());
        pkt[tcp_off + 8..tcp_off + 12].copy_from_slice(&ack.to_be_bytes());
        pkt[tcp_off + 12] = 0x50; // Data offset = 5 (20 bytes)
        pkt[tcp_off + 13] = flags;
        pkt[tcp_off + 14] = 0x40; pkt[tcp_off + 15] = 0x00; // Window = 16384
        pkt[tcp_off + 16] = 0; pkt[tcp_off + 17] = 0;
        pkt[tcp_off + 18] = 0; pkt[tcp_off + 19] = 0;
        pkt[tcp_off + 20..tcp_off + 20 + data.len()].copy_from_slice(data);

        let csum = transport_checksum(my_ip, dst_ip, IP_PROTO_TCP, &pkt[tcp_off..tcp_off + tcp_len]);
        pkt[tcp_off + 16] = (csum >> 8) as u8;
        pkt[tcp_off + 17] = (csum & 0xFF) as u8;

        build_eth_ipv4_header(&mut pkt, gw_mac, my_mac, my_ip, dst_ip, IP_PROTO_TCP, tcp_len);
        let _ = e1000::send_packet(&pkt[..34 + tcp_len]);
    };

    // 1. TCP SYN -> SYN-ACK
    let mut rx = [0u8; 1600];
    let mut connected = false;
    for _ in 0..2 {
        send_tcp(0x02, my_seq, 0, &[]); // SYN
        for _ in 0..450_000 {
            if let Some(len) = e1000::recv_packet(&mut rx) {
                maybe_reply_arp(&rx[..len], my_mac, my_ip);
                if len >= 54 && rx[12] == 0x08 && rx[13] == 0x00 && rx[23] == IP_PROTO_TCP {
                    let ihl = ((rx[14] & 0x0F) as usize) * 4;
                    let toff = 14 + ihl;
                    if toff + 20 <= len {
                        let dp = ((rx[toff + 2] as u16) << 8) | (rx[toff + 3] as u16);
                        let flags = rx[toff + 13];
                        if dp == src_port && (flags & 0x12) == 0x12 {
                            let srv_seq = u32::from_be_bytes([rx[toff + 4], rx[toff + 5], rx[toff + 6], rx[toff + 7]]);
                            my_seq = my_seq.wrapping_add(1);
                            peer_ack = srv_seq.wrapping_add(1);
                            send_tcp(0x10, my_seq, peer_ack, &[]); // ACK
                            connected = true;
                            break;
                        }
                    }
                }
            }
            core::hint::spin_loop();
        }
        if connected { break; }
    }
    if !connected {
        return Err("TCP connection timed out (port 80 unreachable)");
    }

    // 2. Send HTTP/1.0 GET Request
    let mut req = [0u8; 256];
    let mut rlen = 0usize;
    let parts: &[&[u8]] = &[
        b"GET ",
        path.as_bytes(),
        b" HTTP/1.0\r\nHost: ",
        host.as_bytes(),
        b"\r\nUser-Agent: SrikeshOS/1.0\r\nConnection: close\r\n\r\n",
    ];
    for p in parts {
        for &b in *p {
            if rlen < req.len() {
                req[rlen] = b;
                rlen += 1;
            }
        }
    }
    send_tcp(0x18, my_seq, peer_ack, &req[..rlen]); // PSH + ACK
    my_seq = my_seq.wrapping_add(rlen as u32);

    // 3. Receive HTTP Response Segments
    let mut total = 0usize;
    let mut idle = 0u32;
    while idle < 900_000 && total < out.len() {
        if let Some(len) = e1000::recv_packet(&mut rx) {
            maybe_reply_arp(&rx[..len], my_mac, my_ip);
            if len >= 54 && rx[12] == 0x08 && rx[13] == 0x00 && rx[23] == IP_PROTO_TCP {
                let ip_total = (((rx[16] as u16) << 8) | (rx[17] as u16)) as usize;
                let ihl = ((rx[14] & 0x0F) as usize) * 4;
                let toff = 14 + ihl;
                if toff + 20 <= len {
                    let dp = ((rx[toff + 2] as u16) << 8) | (rx[toff + 3] as u16);
                    if dp == src_port {
                        let tcp_hdr_len = ((rx[toff + 12] >> 4) as usize) * 4;
                        let flags = rx[toff + 13];
                        let payload_start = toff + tcp_hdr_len;
                        let ip_end = (14 + ip_total).min(len);
                        if payload_start < ip_end {
                            let data_len = ip_end - payload_start;
                            let copy_n = data_len.min(out.len() - total);
                            out[total..total + copy_n].copy_from_slice(&rx[payload_start..payload_start + copy_n]);
                            total += copy_n;
                            peer_ack = peer_ack.wrapping_add(data_len as u32);
                            send_tcp(0x10, my_seq, peer_ack, &[]); // ACK
                            idle = 0;
                        }
                        if (flags & 0x01) != 0 || (flags & 0x04) != 0 {
                            // FIN or RST
                            peer_ack = peer_ack.wrapping_add(1);
                            send_tcp(0x11, my_seq, peer_ack, &[]); // FIN+ACK
                            break;
                        }
                    }
                }
            }
        } else {
            idle += 1;
            core::hint::spin_loop();
        }
    }

    if total == 0 {
        return Err("Empty HTTP response from remote server");
    }
    Ok((total, dst_ip))
}

/// Builds an ICMP Echo Request (Ping) and transmits it over the network
pub fn send_icmp_ping(target_ip: [u8; 4], seq: u16) -> bool {
    let (src_mac, src_ip) = {
        let nic = E1000.lock();
        (nic.mac_addr, nic.ip_addr)
    };

    let mut packet = [0u8; 98];
    build_eth_ipv4_header(&mut packet, [0xFF; 6], src_mac, src_ip, target_ip, IP_PROTO_ICMP, 64);

    packet[34] = 8; // Echo Request
    packet[35] = 0;
    packet[38] = 0x0A;
    packet[39] = 0xBC;
    packet[40] = (seq >> 8) as u8;
    packet[41] = (seq & 0xFF) as u8;

    let payload = b"SRIKESH_OS_NETWORKING_TELEMETRY_GIGABIT_PRO1000_PING";
    for i in 0..56 {
        packet[42 + i] = if i < payload.len() { payload[i] } else { (i as u8) + 0x40 };
    }
    let icmp_csum = internet_checksum(&packet[34..98]);
    packet[36] = (icmp_csum >> 8) as u8;
    packet[37] = (icmp_csum & 0xFF) as u8;

    e1000::send_packet(&packet[..98])
}

/// Executes boot-time DHCP acquisition & network diagnostic
pub fn test_network_subsystem() {
    serial::write_str("\n[NETWORK] Initializing SRIKESH OS TCP/IP Stack (ARP/DHCP/DNS/TCP/HTTP)...\n");
    if e1000::is_link_up() {
        if run_dhcp() {
            serial::write_str("[NETWORK] DHCP configuration complete.\n");
        } else {
            serial::write_str("[NETWORK] DHCP unavailable — interface remains unconfigured.\n");
        }
    } else {
        serial::write_str("[NETWORK] No active Intel e1000 link detected.\n");
    }
}
