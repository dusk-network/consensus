// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

// RoundUpdate carries the data about the new Round, such as the active
// Provisioners, the BidList, the Seed and the Hash.

use std::fmt;

// TODO: consider replacing most of the fields with a full copy of a the tip.
#[derive(Copy, Clone, Default, Debug)]
#[allow(unused)]
pub struct RoundUpdate {
    pub(crate) round: u64,
    pub(crate) seed: [u8; 32],
    pub(crate) hash: [u8; 32],
}

impl RoundUpdate {
    pub fn new(round: u64) -> Self {
        RoundUpdate {
            round,
            ..Default::default()
        }
    }
}

#[derive(Default, Debug)]
pub struct Header {
    pub version: u8,
    pub height: u64,
    pub timestamp: i64,
    pub gas_limit: u64,
    pub prev_block_hash: [u8; 32],
    pub seed: [u8; 32],
    pub generator_bls_pubkey: [u8; 32], // TODO: size should be 96
    pub state_hash: [u8; 32],
    pub hash: [u8; 32],
}

#[derive(Default, Debug)]
pub struct Block {
    pub header: Header,
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "block height: {}", self.header.height)
    }
}

#[derive(Debug)]
pub enum SelectError {
    Continue,
    Canceled,
    Timeout,
}

#[allow(unused)]
pub enum ConsensusError {
    // TODO: Rename InvalidRoundStep
    InvalidRoundStep,
    InvalidBlock,
    InvalidSignature,
    NotImplemented,
}
