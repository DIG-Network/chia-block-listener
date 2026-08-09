//! Live-mainnet proofs for the peer protocol and the block parser.
//!
//! These tests talk to real Chia mainnet full nodes discovered over DNS, and
//! cross-check what they read against `api.coinset.org` — an source outside the
//! crate under test. That external anchor is the point: a handshake failure and
//! a parse failure both surface as "no coins", so an assertion written only
//! against this crate's own output could not tell a working listener from a
//! broken one.
//!
//! ## A measured limitation of today's mainnet peer population
//!
//! Mainnet peers frequently close the socket with WebSocket code 1002 instead
//! of answering `request_block`. The peers that do answer return correct
//! blocks, so this is a connection-acceptance problem rather than a parsing or
//! framing fault.
//!
//! The pattern in the measurements points at this crate's TLS identity: a first
//! request on a freshly connected peer usually succeeds, while requests made
//! after the same process has already connected to that peer, or while a second
//! connection to it is open, are refused. `tls::load_or_generate_cert` persists
//! ONE certificate, so every connection this crate makes presents the same Chia
//! node id — and a pool built with an event sink opens two simultaneous
//! connections per peer with it. That is the leading explanation, not a proven
//! one; it is filed rather than asserted (DIG-Network/dig_ecosystem#2395).
//!
//! The practical consequence for these tests: run them ONE AT A TIME. Each
//! passes on its own; run back to back in a single process they exhaust the
//! peers they have already contacted.
//!
//! `recovers_when_a_peer_is_dropped` FAILS against mainnet today for that same
//! reason compounded by two pool defects: the pool evicts a peer permanently on
//! a single refusal, and its retry loop selects a peer that the request
//! processor then ignores in favour of its own round-robin choice
//! (DIG-Network/dig_ecosystem#2394). The test is kept, unweakened, as the
//! regression test for that fix.
//!
//! They are `#[ignore]`d because they require network access and real peers.
//! Run them with:
//!
//! ```text
//! cargo test --test live_mainnet -- --ignored --nocapture
//! ```

use std::collections::BTreeSet;
use std::time::Duration;

use chia_block_listener::{
    peer_pool::ChiaPeerPool,
    types::{CoinRecord, Event},
    DnsDiscoveryClient,
};
use tokio_util::sync::CancellationToken;

const MAINNET: &str = "mainnet";
const COINSET: &str = "https://api.coinset.org";

/// A mainnet transaction block pinned as the parser fixture.
///
/// Chosen because it is a *transaction* block with a non-trivial generator —
/// 17 additions and 9 removals per coinset — so a parser that silently yields
/// nothing cannot pass. An empty or reward-only block would be exactly the
/// blind fixture this test exists to avoid.
const FIXTURE_HEIGHT: u64 = 6_000_002;

/// How far the peer-reported peak may trail the coinset-reported peak before we
/// call it "not tracking". Mainnet produces a block roughly every 18.75 s and
/// the two observations are seconds apart, so a few blocks of skew is normal
/// and anything beyond it is not.
const PEAK_SKEW_TOLERANCE: u32 = 8;

/// Coin identity as a comparable triple, normalised across the two sources.
///
/// This crate emits bare hex; coinset emits `0x`-prefixed hex and the amount as
/// a JSON number. Comparing the triple rather than a computed coin id keeps the
/// assertion readable and still uniquely identifies a coin.
type CoinKey = (String, String, u64);

fn key_from_record(record: &CoinRecord) -> CoinKey {
    (
        normalize_hex(&record.parent_coin_info),
        normalize_hex(&record.puzzle_hash),
        record
            .amount
            .parse()
            .expect("this crate emits coin amounts as decimal digits"),
    )
}

fn normalize_hex(value: &str) -> String {
    value.trim_start_matches("0x").to_ascii_lowercase()
}

async fn coinset(endpoint: &str, body: serde_json::Value) -> serde_json::Value {
    let client = reqwest::Client::builder()
        .user_agent("chia-block-listener-live-test")
        .build()
        .expect("client builds");
    client
        .post(format!("{COINSET}/{endpoint}"))
        .json(&body)
        .send()
        .await
        .unwrap_or_else(|e| panic!("coinset {endpoint} request failed: {e}"))
        .json()
        .await
        .unwrap_or_else(|e| panic!("coinset {endpoint} returned unparseable JSON: {e}"))
}

/// Independently observed mainnet peak, used to anchor the peer-reported peak.
async fn coinset_peak_height() -> u32 {
    let state = coinset("get_blockchain_state", serde_json::json!({})).await;
    state["blockchain_state"]["peak"]["height"]
        .as_u64()
        .expect("coinset reports a peak height") as u32
}

/// Independently observed additions and removals for one block.
async fn coinset_additions_and_removals(height: u64) -> (BTreeSet<CoinKey>, BTreeSet<CoinKey>) {
    let record = coinset(
        "get_block_record_by_height",
        serde_json::json!({ "height": height }),
    )
    .await;
    let header_hash = record["block_record"]["header_hash"]
        .as_str()
        .expect("coinset reports a header hash")
        .to_string();

    let payload = coinset(
        "get_additions_and_removals",
        serde_json::json!({ "header_hash": header_hash }),
    )
    .await;

    (
        coin_keys(&payload["additions"]),
        coin_keys(&payload["removals"]),
    )
}

fn coin_keys(records: &serde_json::Value) -> BTreeSet<CoinKey> {
    records
        .as_array()
        .expect("coinset returns an array of coin records")
        .iter()
        .map(|record| {
            let coin = &record["coin"];
            (
                normalize_hex(coin["parent_coin_info"].as_str().expect("parent is hex")),
                normalize_hex(coin["puzzle_hash"].as_str().expect("puzzle hash is hex")),
                coin["amount"].as_u64().expect("amount is a number"),
            )
        })
        .collect()
}

/// Connects to mainnet peers discovered over DNS, returning a pool with at
/// least `wanted` live, handshaken peers.
///
/// Every peer here has completed a real Chia handshake — `add_peer` connects,
/// sends the handshake and validates the response before it returns — so a
/// non-empty pool *is* the handshake proof.
async fn connect_mainnet_pool(wanted: usize) -> ChiaPeerPool {
    connect_mainnet_pool_skipping(wanted, 0).await
}

/// As [`connect_mainnet_pool`], but ignoring the first `skip` reachable peers.
///
/// Callers that must retry against a *different* peer use this. Around nine in
/// ten mainnet nodes today refuse `request_block` outright (see the module docs
/// on that limitation), so a block-fetch proof that binds itself to one peer
/// measures that peer's policy rather than this crate's correctness.
async fn connect_mainnet_pool_skipping(wanted: usize, skip: usize) -> ChiaPeerPool {
    let discovery = DnsDiscoveryClient::new()
        .await
        .expect("DNS resolver initialises");
    let discovered = discovery
        .discover_mainnet_peers()
        .await
        .expect("mainnet introducers resolve");

    // IPv6 first, IPv4 as the fallback, matching the ecosystem's peer-transport
    // rule. Both families are capped separately: concatenating and truncating
    // would hand an IPv4-only host a list of nothing but unreachable IPv6
    // addresses, which reads as "no peers on mainnet" rather than "no IPv6
    // here". Peers are taken in resolver order rather than shuffled so that a
    // failure is reproducible.
    const PER_FAMILY_ATTEMPTS: usize = 60;
    let candidates: Vec<_> = discovered
        .ipv6_peers
        .iter()
        .take(PER_FAMILY_ATTEMPTS)
        .chain(discovered.ipv4_peers.iter().take(PER_FAMILY_ATTEMPTS))
        .cloned()
        .collect();
    assert!(
        !candidates.is_empty(),
        "DNS discovery returned no mainnet peers"
    );

    // The pool is built with an event sink because that is what starts the
    // streaming listener; without it the pool is a request/response client and
    // never observes a peer's peak announcements at all. Events are drained and
    // discarded — these tests read pool state, not the stream.
    let (event_tx, mut event_rx) = tokio::sync::mpsc::channel::<Event>(256);
    tokio::spawn(async move { while event_rx.recv().await.is_some() {} });
    let pool = ChiaPeerPool::new_with_event_sink(event_tx, CancellationToken::new());

    let mut connected = 0;
    let mut skipped = 0;
    let mut failures = Vec::new();

    for peer in candidates {
        match tokio::time::timeout(
            Duration::from_secs(15),
            pool.add_peer(peer.host.to_string(), peer.port, MAINNET.to_string()),
        )
        .await
        {
            Ok(Ok(peer_id)) => {
                if skipped < skip {
                    skipped += 1;
                    let _ = pool.remove_peer(peer_id).await;
                    continue;
                }
                connected += 1;
                if connected >= wanted {
                    return pool;
                }
            }
            Ok(Err(e)) => failures.push(format!("{}:{} -> {e}", peer.host, peer.port)),
            Err(_) => failures.push(format!("{}:{} -> timeout", peer.host, peer.port)),
        }
    }

    panic!(
        "connected to {connected} of {wanted} required mainnet peers; failures:\n{}",
        failures.join("\n")
    );
}

/// Proof 1: DNS discovery finds mainnet peers and the Chia handshake completes.
#[tokio::test]
#[ignore = "requires live mainnet peers"]
async fn handshakes_with_live_mainnet_peers() {
    let pool = connect_mainnet_pool(1).await;
    let peers = pool
        .get_connected_peers()
        .await
        .expect("pool reports peers");
    assert!(
        !peers.is_empty(),
        "a handshaken peer must appear in the connected set"
    );
    pool.shutdown_and_wait().await.expect("pool shuts down");
}

/// Proof 2: the peak the peers report tracks the peak an outside source reports.
///
/// The external anchor is what makes this load-bearing. Asserting only that the
/// height is non-zero, or that it advances, would pass on a peer that is stuck
/// thousands of blocks behind — which is indistinguishable from a healthy
/// listener if you never look outside the crate.
#[tokio::test]
#[ignore = "requires live mainnet peers"]
async fn tracks_the_true_mainnet_peak() {
    let pool = connect_mainnet_pool(2).await;

    // Peaks arrive as unsolicited NewPeakWallet messages, so give the peers a
    // block interval or so to announce one.
    let mut observed = None;
    for _ in 0..12 {
        if let Some(peak) = pool.get_highest_peak().await {
            observed = Some(peak);
            break;
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
    let observed = observed.expect("peers announced a peak within 60s");

    let anchor = coinset_peak_height().await;
    let skew = anchor.abs_diff(observed);
    assert!(
        skew <= PEAK_SKEW_TOLERANCE,
        "peer-reported peak {observed} is {skew} blocks from the independently \
         observed peak {anchor}, beyond the {PEAK_SKEW_TOLERANCE}-block tolerance"
    );

    pool.shutdown_and_wait().await.expect("pool shuts down");
}

/// Fetches the fixture block, trying successive peers until one serves it.
///
/// The retry is about peer policy, not about flakiness in the code under test:
/// most mainnet nodes close the connection rather than answer `request_block`.
async fn fetch_fixture_block_from_any_willing_peer(
) -> chia_block_listener::types::BlockReceivedEvent {
    const MAX_PEERS_TO_ASK: usize = 30;
    let mut refusals = Vec::new();

    for skip in 0..MAX_PEERS_TO_ASK {
        let pool = connect_mainnet_pool_skipping(1, skip).await;
        let outcome = tokio::time::timeout(
            Duration::from_secs(60),
            pool.get_block_by_height(FIXTURE_HEIGHT),
        )
        .await
        .expect("block request completes");
        pool.shutdown_and_wait().await.expect("pool shuts down");

        match outcome {
            Ok(block) => return block,
            Err(e) => refusals.push(e.to_string()),
        }
    }

    panic!(
        "no peer out of {MAX_PEERS_TO_ASK} served block {FIXTURE_HEIGHT}; refusals:
{}",
        refusals.join(
            "
"
        )
    );
}

/// Proof 3: a real mainnet block parses to the coin set an outside source
/// reports for that same block.
///
/// This is the assertion that a generator-semantics regression would fail. The
/// parser swallows CLVM errors and returns empty vectors, so a misparse looks
/// exactly like an empty block — comparing against externally obtained coins is
/// the only way to tell those apart.
#[tokio::test]
#[ignore = "requires live mainnet peers"]
async fn parses_a_real_block_into_the_coins_the_chain_actually_has() {
    let block = fetch_fixture_block_from_any_willing_peer().await;

    assert_eq!(block.height as u64, FIXTURE_HEIGHT);
    assert!(
        block.has_transactions_generator,
        "the fixture must be a generator block, or it proves nothing about parsing"
    );

    let (expected_additions, expected_removals) =
        coinset_additions_and_removals(FIXTURE_HEIGHT).await;
    assert!(
        expected_removals.len() >= 5,
        "the fixture must remove several coins, or an all-empty parse would pass"
    );

    let parsed_additions: BTreeSet<CoinKey> =
        block.coin_additions.iter().map(key_from_record).collect();
    let parsed_removals: BTreeSet<CoinKey> =
        block.coin_removals.iter().map(key_from_record).collect();

    assert_eq!(
        parsed_removals, expected_removals,
        "parsed removals disagree with the chain"
    );
    assert_eq!(
        parsed_additions, expected_additions,
        "parsed additions disagree with the chain"
    );
}

/// Proof 4: losing a peer does not lose the block — the pool fails over.
///
/// One peer is removed while two remain reachable, so the surviving peer is a
/// truthful control: if the request still succeeds, failover ran; if the pool
/// had no fallback the request would fail outright.
#[tokio::test]
#[ignore = "requires live mainnet peers"]
async fn recovers_when_a_peer_is_dropped() {
    let pool = connect_mainnet_pool(3).await;

    let peers = pool
        .get_connected_peers()
        .await
        .expect("pool reports peers");
    assert!(peers.len() >= 3, "failover needs a surviving peer to reach");

    let removed = pool
        .remove_peer(peers[0].clone())
        .await
        .expect("peer removal succeeds");
    assert!(removed, "the peer we asked to remove was actually removed");

    let block = tokio::time::timeout(
        Duration::from_secs(60),
        pool.get_block_by_height(FIXTURE_HEIGHT),
    )
    .await
    .expect("block request completes after a peer was dropped")
    .expect("a surviving peer serves the block");

    assert_eq!(block.height as u64, FIXTURE_HEIGHT);
    pool.shutdown_and_wait().await.expect("pool shuts down");
}
