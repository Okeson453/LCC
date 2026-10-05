// The workspace denies `clippy::unwrap_used`, `clippy::expect_used` and
// `clippy::panic` to keep production code free of panicking shortcuts.
// `cargo clippy --all-targets` — which CI runs — also lints integration-test
// targets, and in a test those are the assertion mechanism. Exempt this file
// rather than rewriting every assertion into a `match`.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! Realtime contract conformance for `lcc-realtime-svc`.
//!
//! `Contract/realtime/lcc-realtime-contract.yaml` is the authoritative
//! description of the five dashboard channels and the 19 event names they
//! carry. `realtime-svc` encodes the same thing in Rust
//! (`domain::Channel::{id, allowed_events}` and `events::route_event`).
//!
//! Nothing checked the two against each other. The audit recorded
//! "0/5 realtime channels implemented end-to-end" — and even after the
//! Phase-C implementation landed, a contract event that was added, renamed or
//! removed would have silently stopped being routed, because the Rust
//! `allowed_events` table is just another hand-maintained list.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

use lcc_realtime_svc::domain::Channel;

fn contract_path() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    let mut dir = PathBuf::from(manifest_dir.as_str());
    loop {
        let candidate = dir.join("Contract");
        if candidate.join("realtime").is_dir() {
            return candidate.join("realtime/lcc-realtime-contract.yaml");
        }
        if !dir.pop() {
            panic!("could not locate Contract/ from {manifest_dir}");
        }
    }
}

fn contract_source() -> String {
    fs::read_to_string(contract_path()).unwrap_or_else(|e| panic!("read realtime contract: {e}"))
}

/// Parse `channels:` from the contract into `(id, url_ws, url_sse, events)`.
///
/// The contract is a small, regular YAML subset, so this walks the block
/// rather than pulling in a parser dependency; it is a test helper.
fn contract_channels() -> Vec<ContractChannel> {
    let src = contract_source();
    let mut out: Vec<ContractChannel> = Vec::new();
    let mut cur: Option<ContractChannel> = None;
    let mut in_events = false;

    for line in src.lines() {
        let trimmed = line.trim();

        if let Some(rest) = trimmed.strip_prefix("- id: ") {
            if let Some(c) = cur.take() {
                out.push(c);
            }
            cur = Some(ContractChannel {
                id: rest.trim().to_string(),
                ..ContractChannel::default()
            });
            in_events = false;
            continue;
        }
        let Some(c) = cur.as_mut() else { continue };

        if trimmed == "events:" {
            in_events = true;
            continue;
        }
        if in_events {
            if let Some(ev) = trimmed.strip_prefix("- ") {
                c.events.insert(ev.trim().to_string());
                continue;
            }
            in_events = false;
        }
        if let Some(v) = trimmed.strip_prefix("url_ws:") {
            c.url_ws = v.trim().to_string();
        } else if let Some(v) = trimmed.strip_prefix("url_sse:") {
            c.url_sse = v.trim().to_string();
        }
    }
    if let Some(c) = cur {
        out.push(c);
    }
    out
}

#[derive(Default)]
struct ContractChannel {
    id: String,
    url_ws: String,
    url_sse: String,
    events: BTreeSet<String>,
}

const ALL_CHANNELS: [Channel; 5] = [
    Channel::Briefing,
    Channel::Approvals,
    Channel::Engagement,
    Channel::Compliance,
    Channel::Sequence,
];

#[test]
fn every_contract_channel_is_implemented() {
    let contract = contract_channels();
    assert_eq!(
        contract.len(),
        5,
        "the canonical realtime contract declares 5 dashboard channels"
    );

    for c in &contract {
        assert!(
            ALL_CHANNELS.iter().any(|ch| ch.id() == c.id),
            "contract channel {} has no Channel variant in realtime-svc",
            c.id
        );
    }
}

#[test]
fn every_implemented_channel_is_in_the_contract() {
    let ids: BTreeSet<String> = contract_channels().into_iter().map(|c| c.id).collect();

    for ch in ALL_CHANNELS {
        assert!(
            ids.contains(ch.id()),
            "realtime-svc implements channel {} which the contract does not declare",
            ch.id()
        );
    }
}

#[test]
fn channel_urls_match_the_contract() {
    for c in contract_channels() {
        let Some(ch) = ALL_CHANNELS.iter().find(|ch| ch.id() == c.id) else {
            continue;
        };
        // The id is the `ws.<name>` form, so the urls are derivable from it.
        let name = c.id.strip_prefix("ws.").unwrap_or(&c.id);
        assert_eq!(
            c.url_ws,
            format!("/api/v1/ws/{name}"),
            "ws url drift for {}",
            c.id
        );
        assert_eq!(
            c.url_sse,
            format!("/api/v1/sse/{name}"),
            "sse url drift for {}",
            c.id
        );
        assert_eq!(
            ch.id(),
            c.id,
            "Channel::id() must equal the contract channel id"
        );
    }
}

#[test]
fn event_names_match_the_contract_exactly() {
    let by_channel: BTreeMap<String, BTreeSet<String>> = contract_channels()
        .into_iter()
        .map(|c| (c.id, c.events))
        .collect();

    for ch in ALL_CHANNELS {
        let expected = by_channel
            .get(ch.id())
            .unwrap_or_else(|| panic!("contract has no events for channel {}", ch.id()));
        let actual: BTreeSet<String> = ch.allowed_events().iter().map(|s| s.to_string()).collect();
        assert_eq!(
            &actual,
            expected,
            "realtime-svc's allowed_events for {} does not match the contract",
            ch.id()
        );
    }
}

#[test]
fn every_contract_event_routes_to_its_channel() {
    // `route_event` is what the Redis consumer uses to pick a channel. An
    // event the contract declares must be routable, otherwise it would be
    // published and then dropped.
    for c in contract_channels() {
        for ev in &c.events {
            assert!(
                lcc_realtime_svc::events::route_event(ev).is_some(),
                "contract event {ev} (channel {}) is not routable by realtime-svc",
                c.id
            );
        }
    }
}

#[test]
fn the_contract_declares_nineteen_events() {
    let total: usize = contract_channels().iter().map(|c| c.events.len()).sum();
    assert_eq!(
        total, 19,
        "the canonical realtime contract declares 19 events across 5 channels"
    );
}
