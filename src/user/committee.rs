// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::user::provisioners::Provisioners;
use crate::user::sortition;

use crate::config;
use crate::util::cluster::Cluster;
use crate::util::pubkey::ConsensusPublicKey;
use std::collections::{BTreeMap, HashMap};
use std::mem;

#[allow(unused)]
#[derive(Default, Debug, Clone)]
pub struct Committee {
    members: BTreeMap<ConsensusPublicKey, usize>,
    this_member_key: ConsensusPublicKey,
    cfg: sortition::Config,
    total: usize,
}

#[allow(unused)]
impl Committee {
    /// Generates a new committee from the given provisioners state and sortition config.
    ///
    /// It executes deterministic sortition algorithm.
    ///
    /// # Arguments
    /// * `pubkey_bls` - This is the BLS public key of the (this node) provisioner running the consensus. It is mainly used in `am_member` method.
    pub fn new(
        pubkey_bls: ConsensusPublicKey,
        provisioners: &mut Provisioners,
        cfg: sortition::Config,
    ) -> Self {
        provisioners.update_eligibility_flag(cfg.round);
        // Generate committee using deterministic sortition.
        let res = provisioners.create_committee(&cfg);
        let max_committee_size = cfg.max_committee_size;

        // Turn the raw vector into a hashmap where we map a pubkey to its occurrences.
        let mut committee = Self {
            members: BTreeMap::new(),
            this_member_key: pubkey_bls,
            cfg,
            total: 0,
        };

        for member_key in res {
            *committee.members.entry(member_key).or_insert(0) += 1;
            committee.total += 1;
        }

        debug_assert!(
            committee.total
                == provisioners.get_eligible_size(max_committee_size)
        );

        committee
    }

    /// Returns true if `pubkey_bls` is a member of the generated committee.
    pub fn is_member(&self, pubkey_bls: &ConsensusPublicKey) -> bool {
        self.members.contains_key(pubkey_bls)
    }

    /// Returns true if `my pubkey` is a member of the generated committee.
    pub fn am_member(&self) -> bool {
        self.is_member(&self.this_member_key)
    }

    /// Returns this provisioner BLS public key.
    pub fn get_my_pubkey(&self) -> &ConsensusPublicKey {
        &self.this_member_key
    }

    pub fn votes_for(&self, pubkey_bls: &ConsensusPublicKey) -> Option<usize> {
        self.members.get(pubkey_bls).copied()
    }

    // get_occurrences returns values in a vec
    pub fn get_occurrences(&self) -> Vec<usize> {
        self.members.clone().into_values().collect()
    }

    /// Returns number of unique members of the generated committee.
    pub fn size(&self) -> usize {
        self.members.len()
    }

    /// Returns target quorum for the generated committee.
    pub fn quorum(&self) -> usize {
        let size = self.total as f64;
        (size * config::CONSENSUS_QUORUM_THRESHOLD).ceil() as usize
    }

    pub fn bits(&self, voters: &Cluster<ConsensusPublicKey>) -> u64 {
        let mut bits: u64 = 0;

        debug_assert!(self.members.len() <= mem::size_of_val(&bits) * 8);

        let mut pos = 0;
        for (pk, _) in voters.iter() {
            for (pos, (member_pk, _)) in self.members.iter().enumerate() {
                if member_pk.eq(pk) {
                    bits |= 1 << pos; // flip the i-th bit to 1
                    break;
                }
            }
        }

        bits
    }

    /// Intersects the bit representation of a VotingCommittee subset with the whole VotingCommittee set.
    pub fn intersect(&self, bitset: u64) -> Cluster<ConsensusPublicKey> {
        if bitset == 0 {
            return Cluster::<ConsensusPublicKey>::new();
        }

        let mut a = Cluster::new();
        for (pos, (member_pk, weight)) in self.members.iter().enumerate() {
            if ((bitset >> pos) & 1) != 0 {
                a.set_weight(member_pk, *weight);
            }
        }
        a
    }

    pub fn total_occurrences(
        &self,
        voters: &Cluster<ConsensusPublicKey>,
    ) -> usize {
        let mut total = 0;
        for (item_pk, _) in voters.iter() {
            if let Some(weight) = self.votes_for(item_pk) {
                total += weight;
            };
        }

        total
    }
}

/// Implements a cache of generated committees so that they can be reused.
///
/// This is useful in Agreement step where messages from different steps per a single round are concurrently processed.
/// A committee is uniquely represented by sortition::Config.
pub struct CommitteeSet {
    committees: HashMap<sortition::Config, Committee>,
    provisioners: Provisioners,
    this_member_key: ConsensusPublicKey,
}

impl CommitteeSet {
    pub fn new(pubkey: ConsensusPublicKey, provisioners: Provisioners) -> Self {
        CommitteeSet {
            provisioners,
            committees: HashMap::new(),
            this_member_key: pubkey,
        }
    }

    pub fn is_member(
        &mut self,
        pubkey: &ConsensusPublicKey,
        cfg: &sortition::Config,
    ) -> bool {
        self.get_or_create(cfg).is_member(pubkey)
    }

    pub fn votes_for(
        &mut self,
        pubkey: &ConsensusPublicKey,
        cfg: &sortition::Config,
    ) -> Option<usize> {
        self.get_or_create(cfg).votes_for(pubkey)
    }

    pub fn quorum(&mut self, cfg: &sortition::Config) -> usize {
        self.get_or_create(cfg).quorum()
    }

    pub fn intersect(
        &mut self,
        bitset: u64,
        cfg: &sortition::Config,
    ) -> Cluster<ConsensusPublicKey> {
        self.get_or_create(cfg).intersect(bitset)
    }

    pub fn total_occurrences(
        &mut self,
        voters: &Cluster<ConsensusPublicKey>,
        cfg: &sortition::Config,
    ) -> usize {
        self.get_or_create(cfg).total_occurrences(voters)
    }

    pub fn get_provisioners(&self) -> &Provisioners {
        &self.provisioners
    }

    pub fn bits(
        &mut self,
        voters: &Cluster<ConsensusPublicKey>,
        cfg: &sortition::Config,
    ) -> u64 {
        self.get_or_create(cfg).bits(voters)
    }

    fn get_or_create(&mut self, cfg: &sortition::Config) -> &Committee {
        self.committees
            .entry(cfg.clone())
            .or_insert_with_key(|config| {
                Committee::new(
                    self.this_member_key.clone(),
                    &mut self.provisioners,
                    config.clone(),
                )
            })
    }
}
