// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use consensus::user::committee::Committee;
use consensus::user::provisioners::{Provisioners, DUSK};
use consensus::user::sortition::Config;
use consensus::util::pubkey::ConsensusPublicKey;

use hex::FromHex;

#[test]
fn test_deterministic_sortition_1() {
    // Create provisioners with bls keys read from an external file.
    let mut p = generate_provisioners(5);

    // Execute sortition with specific config
    let cfg = Config::new([0; 32], 1, 1, 64);
    p.update_eligibility_flag(cfg.round);

    assert_eq!(
        vec![2, 1, 1],
        Committee::new(ConsensusPublicKey::default(), &mut p, cfg).get_occurrences()
    );
}

#[test]
fn test_deterministic_sortition_2() {
    // Create provisioners with bls keys read from an external file.
    let mut p = generate_provisioners(5);

    let cfg = Config::new(
        <[u8; 32]>::from_hex("b70189c7e7a347989f4fbc1205ce612f755dfc489ecf28f9f883800acf078bd5")
            .unwrap_or([0; 32]),
        7777,
        8,
        45,
    );
    p.update_eligibility_flag(cfg.round);

    assert_eq!(
        vec![1, 3],
        Committee::new(ConsensusPublicKey::default(), &mut p, cfg).get_occurrences()
    );
}

#[test]
fn test_quorum() {
    // Create provisioners with bls keys read from an external file.
    let mut p = generate_provisioners(5);

    let cfg = Config::new(
        <[u8; 32]>::from_hex("b70189c7e7a347989f4fbc1205ce612f755dfc489ecf28f9f883800acf078bd5")
            .unwrap_or([0; 32]),
        7777,
        8,
        64,
    );
    p.update_eligibility_flag(cfg.round);

    let c = Committee::new(ConsensusPublicKey::default(), &mut p, cfg);
    assert_eq!(c.quorum(), 3);
}

#[test]
fn test_quorum_max_size() {
    // Create provisioners with bls keys read from an external file.
    let mut p = generate_provisioners(5);

    let cfg = Config::new(
        <[u8; 32]>::from_hex("b70189c7e7a347989f4fbc1205ce612f755dfc489ecf28f9f883800acf078bd5")
            .unwrap_or([0; 32]),
        7777,
        8,
        4,
    );
    p.update_eligibility_flag(cfg.round);

    let c = Committee::new(ConsensusPublicKey::default(), &mut p, cfg);
    assert_eq!(c.quorum(), 3);
}

#[test]
fn test_intersect() {
    let mut p = generate_provisioners(10);

    let cfg = Config::new([0; 32], 1, 3, 200);
    p.update_eligibility_flag(cfg.round);
    // println!("{:#?}", p);

    let c = Committee::new(ConsensusPublicKey::default(), &mut p, cfg);
    // println!("{:#?}", c);

    let max_bitset = (2_i32.pow((c.size()) as u32) - 1) as u64;
    println!("max_bitset: {} / {:#064b} ", max_bitset, max_bitset);

    for bitset in 0..max_bitset {
        //println!("bitset: {:#064b}", bitset);
        let result = c.intersect(bitset);
        assert_eq!(c.bits(&result), bitset, "testing with  bitset:{}", bitset);
    }
}

fn generate_provisioners(n: usize) -> Provisioners {
    let mut p = Provisioners::new();
    for i in 1..n {
        let stake_value = 1000 * (i as u64) * DUSK;
        p.add_member_with_value(ConsensusPublicKey::from_sk_seed_u64(i as u64), stake_value);
    }
    p
}
