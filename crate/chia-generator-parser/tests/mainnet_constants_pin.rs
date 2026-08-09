//! Pins the upstream `chia_consensus::TEST_CONSTANTS` fixture to Chia mainnet values.
//!
//! # Why this test exists
//!
//! `GeneratorParser::process_generator_for_coins` runs mainnet block generators
//! against `TEST_CONSTANTS`. Despite the name, that upstream struct carries the
//! real Chia **mainnet** genesis challenge and mainnet cost limits — which is the
//! only reason parsing mainnet blocks with it is correct.
//!
//! It is nonetheless an upstream *test fixture* with no stability contract: a
//! patch-level `chia-consensus` bump could change its genesis with no semver
//! signal. That would not be a compile error. It would make
//! `check_agg_sig_unsafe_message` (called unconditionally by `chia-consensus`,
//! outside every `DONT_VALIDATE_SIGNATURE` guard) reject on the wrong suffixes,
//! the resulting `ValidationErr` would be swallowed by the parser's error arms,
//! and mainnet blocks would silently parse to **zero coins**.
//!
//! This test converts that silent wrong answer into a red build.
//!
//! # Why the expectations are built, not transcribed
//!
//! The root (`agg_sig_me_additional_data`) is taken from `dig_constants`, a
//! source independent of the crate under test, and the six derived suffixes are
//! recomputed from that independent root. Deriving them from
//! `TEST_CONSTANTS.agg_sig_me_additional_data` would let an upstream genesis
//! change move the root and all six derivatives consistently, leaving this test
//! green while proving nothing.

use chia_consensus::consensus_constants::TEST_CONSTANTS;
use dig_constants::CHIA_L1_MAINNET_AGG_SIG_ME;
use sha2::{Digest, Sha256};

/// Chia mainnet `cost_per_byte`
/// (`chia-consensus-0.36.1/src/consensus_constants.rs`, `TEST_CONSTANTS`;
/// identical to `chia_sdk_types::MAINNET_CONSTANTS`). No `dig-constants` home.
const MAINNET_COST_PER_BYTE: u64 = 12_000;

/// Chia mainnet `max_block_cost_clvm` (same source as above).
const MAINNET_MAX_BLOCK_COST_CLVM: u64 = 11_000_000_000;

/// Derives an `AGG_SIG_*_ADDITIONAL_DATA` suffix from the network's AGG_SIG_ME
/// root, per Chia's rule `sha256(agg_sig_me_additional_data || opcode_byte)`.
///
/// Verified against every literal in `chia-consensus-0.36.1`'s `TEST_CONSTANTS`;
/// the opcode bytes are `chia-consensus-0.36.1/src/opcodes.rs:7-12`.
fn derive_additional_data(root: &[u8; 32], condition_opcode: u8) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(root);
    hasher.update([condition_opcode]);
    hasher.finalize().into()
}

const AGG_SIG_PARENT: u8 = 43;
const AGG_SIG_PUZZLE: u8 = 44;
const AGG_SIG_AMOUNT: u8 = 45;
const AGG_SIG_PUZZLE_AMOUNT: u8 = 46;
const AGG_SIG_PARENT_AMOUNT: u8 = 47;
const AGG_SIG_PARENT_PUZZLE: u8 = 48;

#[test]
fn test_constants_agg_sig_me_is_chia_mainnet_genesis() {
    assert_eq!(
        TEST_CONSTANTS.agg_sig_me_additional_data.as_ref(),
        CHIA_L1_MAINNET_AGG_SIG_ME.as_slice(),
        "upstream TEST_CONSTANTS no longer carries the Chia mainnet genesis; \
         parsing mainnet blocks with it is no longer correct"
    );
}

#[test]
fn test_constants_derived_agg_sig_suffixes_match_mainnet() {
    let root = &CHIA_L1_MAINNET_AGG_SIG_ME;

    for (label, opcode, actual) in [
        (
            "agg_sig_parent",
            AGG_SIG_PARENT,
            TEST_CONSTANTS.agg_sig_parent_additional_data,
        ),
        (
            "agg_sig_puzzle",
            AGG_SIG_PUZZLE,
            TEST_CONSTANTS.agg_sig_puzzle_additional_data,
        ),
        (
            "agg_sig_amount",
            AGG_SIG_AMOUNT,
            TEST_CONSTANTS.agg_sig_amount_additional_data,
        ),
        (
            "agg_sig_puzzle_amount",
            AGG_SIG_PUZZLE_AMOUNT,
            TEST_CONSTANTS.agg_sig_puzzle_amount_additional_data,
        ),
        (
            "agg_sig_parent_amount",
            AGG_SIG_PARENT_AMOUNT,
            TEST_CONSTANTS.agg_sig_parent_amount_additional_data,
        ),
        (
            "agg_sig_parent_puzzle",
            AGG_SIG_PARENT_PUZZLE,
            TEST_CONSTANTS.agg_sig_parent_puzzle_additional_data,
        ),
    ] {
        assert_eq!(
            actual.as_ref(),
            derive_additional_data(root, opcode).as_slice(),
            "TEST_CONSTANTS.{label}_additional_data drifted from the value derived \
             from the Chia mainnet genesis; mainnet AGG_SIG validation would use \
             the wrong suffix"
        );
    }
}

#[test]
fn test_constants_cost_limits_match_mainnet() {
    assert_eq!(
        TEST_CONSTANTS.cost_per_byte, MAINNET_COST_PER_BYTE,
        "TEST_CONSTANTS.cost_per_byte drifted from the mainnet value; generator \
         cost accounting would diverge from consensus"
    );
    assert_eq!(
        TEST_CONSTANTS.max_block_cost_clvm, MAINNET_MAX_BLOCK_COST_CLVM,
        "TEST_CONSTANTS.max_block_cost_clvm drifted from the mainnet value; valid \
         mainnet blocks could exceed the cost limit and parse to zero coins"
    );
}
