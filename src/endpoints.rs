//! All the API endpoint URL-s
#![allow(unused)]

pub const BASE_URL: &str = "https://rackhost.hu";
pub const LOGIN_PATH: &str = "/site/login";
pub const DNS_ZONE_PATH: &str = "/dnsZone";
pub const DOMAIN_SEARCH_PATH: &str = "/domain"; // followed by "?domainList=<name>"