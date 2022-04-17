/*
 * Copyright (c) 2022  Michal Douša.
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::cmp::max;
use std::cmp::min;
use std::collections::HashSet;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;
use std::ops::RangeInclusive;

/// IP configuration of the MDChat server part.
///
/// # Features
///
///  -  banning specific IP addresses
///  -  banning IP address ranges
///  -  excluding specific IP addresses from ban (but not ranges - intentionally)
pub struct IpFilteringConfig {
    v4_allowed: HashSet<Ipv4Addr>,
    v4_banned: HashSet<Ipv4Addr>,
    v4_banned_ranges: HashSet<RangeInclusive<Ipv4Addr>>,
    v6_allowed: HashSet<Ipv6Addr>,
    v6_banned: HashSet<Ipv6Addr>,
    v6_banned_ranges: HashSet<RangeInclusive<Ipv6Addr>>,
}

impl IpFilteringConfig {
    /// Creates a new empty [`IpConfig`] instance. Using this constructor is same as
    /// using [`Default`]'s implementation.
    pub fn new() -> Self {
        Self {
            v4_allowed: HashSet::new(),
            v4_banned: HashSet::new(),
            v4_banned_ranges: HashSet::new(),
            v6_allowed: HashSet::new(),
            v6_banned: HashSet::new(),
            v6_banned_ranges: HashSet::new(),
        }
    }

    /// Appends another instance to `self`
    ///
    /// # Parameters
    ///
    ///  -  `other`: immutable borrow of [`IpConfig`] instance to clone all IP
    ///     addresses and IP address ranges from
    pub fn append(&mut self, other: &Self) {
        self.v4_allowed = &self.v4_allowed | &other.v4_allowed;
        self.v4_banned = &self.v4_banned | &other.v4_banned;
        self.v4_banned_ranges = &self.v4_banned_ranges | &other.v4_banned_ranges;
        self.v6_allowed = &self.v6_allowed | &other.v6_allowed;
        self.v6_banned = &self.v6_banned | &other.v6_banned;
        self.v6_banned_ranges = &self.v6_banned_ranges | &other.v6_banned_ranges;
    }

    /// Allows certain IP address. Allowing an IP address has always greater
    /// precedence than banning it (either independently or by IP address range).
    ///
    /// # Parameters
    ///
    ///  -  `addr`: IP address to be allowed
    ///
    /// # Return value
    ///
    ///  -  `true` if specified address has been already allowed
    ///  -  `false` otherwise
    pub fn allow(&mut self, addr: IpAddr) -> bool {
        match addr {
            IpAddr::V4(addr) => self.v4_allowed.insert(addr),
            IpAddr::V6(addr) => self.v6_allowed.insert(addr),
        }
    }

    /// Bans certain IP address. Banning an IP address always has lower precedence
    /// than allowing it (either independently or by IP address range).
    ///
    /// # Parameters
    ///
    ///  -  `addr`: IP address to be allowed
    ///
    /// # Return value
    ///
    ///  -  `true` if specified address has been already allowed
    ///  -  `false` otherwise
    pub fn ban(&mut self, addr: IpAddr) -> bool {
        match addr {
            IpAddr::V4(addr) => self.v4_banned.insert(addr),
            IpAddr::V6(addr) => self.v6_banned.insert(addr),
        }
    }

    pub fn ban_range(&mut self, from: IpAddr, to: IpAddr) -> Result<bool, String> {
        match (from, to) {
            (IpAddr::V4(from), IpAddr::V4(to)) => {
                let lower = min(from, to);
                let upper = max(from, to);
                Result::Ok(self.ban_v4_range(lower..=upper))
            },
            (IpAddr::V6(from), IpAddr::V6(to)) => {
                let lower = min(from, to);
                let upper = max(from, to);
                Result::Ok(self.ban_v6_range(lower..=upper))
            },
            other => Result::Err("Bounds of IP address range must be the same version".into())
        }
    }

    /// Bans specified IPv4 address range.
    ///
    /// # Parameters
    ///
    ///  -  `range`: range to be banned
    ///
    /// # Return value
    ///
    ///  -  `true` if specified range has been already banned
    ///  -  `false` otherwise
    pub fn ban_v4_range(&mut self, range: RangeInclusive<Ipv4Addr>) -> bool {
        self.v4_banned_ranges.insert(range)
    }

    /// Bans specified IPv6 address range.
    ///
    /// # Parameters
    ///
    ///  -  `range`: range to be banned
    ///
    /// # Return value
    ///
    ///  -  `true` if specified range has been already banned
    ///  -  `false` otherwise
    pub fn ban_v6_range(&mut self, range: RangeInclusive<Ipv6Addr>) -> bool {
        self.v6_banned_ranges.insert(range)
    }

    /// Returns a [`bool`], if specified IP address is banned (`false`) or not
    /// (`true`).
    ///
    /// # Return value
    ///
    ///  -  `true`, if IP address *is not* banned
    ///  -  `false`, if IP address *is* banned
    //noinspection RsLift
    pub fn is_allowed(&self, addr: &IpAddr) -> bool {
        match addr {
            IpAddr::V4(addr) => {
                if self.v4_allowed.contains(addr) { return true }
                if self.v4_banned.contains(addr) { return false }
                for range in &self.v4_banned_ranges {
                    if range.contains(addr) { return false }
                }
                return true
            },
            IpAddr::V6(addr) => {
                if self.v6_allowed.contains(addr) { return true }
                if self.v6_banned.contains(addr) { return false }
                for range in &self.v6_banned_ranges {
                    if range.contains(addr) { return false }
                }
                return true
            }
        }
    }
}
