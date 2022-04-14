use std::cmp::max;
use std::cmp::min;
use std::collections::HashSet;
use std::net::{IpAddr, Ipv4Addr};
use std::net::Ipv6Addr;
use std::ops::RangeInclusive;

pub struct Ipv4Config {
    allowed: HashSet<u32>,
    banned: HashSet<u32>,
    banned_ranges: HashSet<RangeInclusive<u32>>
}

impl Ipv4Config {
    pub fn new() -> Self {
        Self {
            allowed: HashSet::new(),
            banned: HashSet::new(),
            banned_ranges: HashSet::new()
        }
    }

    pub fn append(&mut self, other: &Self) {
        self.allowed = &self.allowed | &other.allowed;
        self.banned = &self.banned | &other.banned;
        self.banned_ranges = &self.banned_ranges | &other.banned_ranges;
    }

    pub fn allow(&mut self, addr: &Ipv4Addr) {
        self.allowed.insert(u32::from_be_bytes(addr.octets()));
    }

    pub fn ban(&mut self, addr: &Ipv4Addr) {
        self.banned.insert(u32::from_be_bytes(addr.octets()));
    }

    pub fn ban_range(&mut self, from: &Ipv4Addr, to: &Ipv4Addr) {
        let from = u32::from_be_bytes(from.octets());
        let to = u32::from_be_bytes(to.octets());
        let lower = min(from, to);
        let higher = max(from, to);
        self.banned_ranges.insert(lower..=higher);
    }

    pub fn is_allowed_addr(&self, addr: &Ipv4Addr) -> bool {
        let addr = u32::from_be_bytes(addr.octets());
        if self.allowed.contains(&addr) { return true }
        if self.banned.contains(&addr) { return false }
        for range in &self.banned_ranges {
            if range.contains(&addr) { return false }
        }
        return true;
    }
}

pub struct Ipv6Config {
    allowed: HashSet<u128>,
    banned: HashSet<u128>,
    banned_ranges: HashSet<RangeInclusive<u128>>
}

impl Ipv6Config {
    pub fn new() -> Self {
        Self {
            allowed: HashSet::new(),
            banned: HashSet::new(),
            banned_ranges: HashSet::new()
        }
    }

    pub fn append(&mut self, other: &Self) {
        self.allowed = &self.allowed | &other.allowed;
        self.banned = &self.banned | &other.banned;
        self.banned_ranges = &self.banned_ranges | &other.banned_ranges;
    }

    pub fn allow(&mut self, addr: &Ipv6Addr) {
        self.allowed.insert(u128::from_be_bytes(addr.octets()));
    }

    pub fn ban(&mut self, addr: &Ipv6Addr) {
        self.banned.insert(u128::from_be_bytes(addr.octets()));
    }

    pub fn ban_range(&mut self, from: &Ipv6Addr, to: &Ipv6Addr) {
        let from = u128::from_be_bytes(from.octets());
        let to = u128::from_be_bytes(to.octets());
        let lower = min(from, to);
        let higher = max(from, to);
        self.banned_ranges.insert(lower..=higher);
    }

    pub fn is_allowed_addr(&self, addr: &Ipv6Addr) -> bool {
        let addr = u128::from_be_bytes(addr.octets());
        if self.allowed.contains(&addr) { return true }
        if self.banned.contains(&addr) { return false }
        for range in &self.banned_ranges {
            if range.contains(&addr) { return false }
        }
        return true;
    }
}

pub struct IpConfig {
    v4: Ipv4Config,
    v6: Ipv6Config
}

impl IpConfig {
    pub fn new() -> Self {
        Self {
            v4: Ipv4Config::new(),
            v6: Ipv6Config::new()
        }
    }

    pub fn append(&mut self, other: &Self) {
        self.v4.append(&other.v4);
        self.v6.append(&other.v6);
    }

    pub fn allow(&mut self, addr: &IpAddr) {
        match addr {
            IpAddr::V4(addr) => self.v4.allow(addr),
            IpAddr::V6(addr) => self.v6.allow(addr),
        }
    }

    pub fn ban(&mut self, addr: &IpAddr) {
        match addr {
            IpAddr::V4(addr) => self.v4.ban(addr),
            IpAddr::V6(addr) => self.v6.ban(addr),
        }
    }

    pub fn ban_range(&mut self, from: &IpAddr, to: &IpAddr) -> Result<(), String> {
        match (from, to) {
            (IpAddr::V4(from), IpAddr::V4(to)) => Result::Ok(self.v4.ban_range(from, to)),
            (IpAddr::V6(from), IpAddr::V6(to)) => Result::Ok(self.v6.ban_range(from, to)),
            _mixed => Result::Err("Cannot combine IPv4 and IPv6 into a single range".to_string())
        }
    }

    pub fn is_allowed_addr(&self, addr: &IpAddr) -> bool {
        match addr {
            IpAddr::V4(addr) => self.v4.is_allowed_addr(addr),
            IpAddr::V6(addr) => self.v6.is_allowed_addr(addr),
        }
    }
}