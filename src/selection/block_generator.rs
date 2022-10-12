// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::commons::{sign, Block, RoundUpdate};
use crate::messages::payload::NewBlock;
use crate::messages::{Header, Message};
use crate::util::pubkey::PublicKey;
use crate::{commons, config};
use dusk_bytes::Serializable;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub struct Generator {}

impl Generator {
    pub async fn generate_candidate_message(&self, ru: RoundUpdate, step: u8) -> Message {
        let candidate = self
            .generate_block(ru.pubkey_bls, ru.round, ru.seed, ru.hash, ru.timestamp)
            .await;

        let msg_header = Header {
            pubkey_bls: ru.pubkey_bls,
            round: ru.round,
            block_hash: candidate.header.hash,
            step,
        };

        Message::from_newblock(
            msg_header,
            NewBlock {
                prev_hash: [0; 32],
                candidate,
                signed_hash: sign(ru.secret_key, ru.pubkey_bls.to_bls_pk(), msg_header),
            },
        )
    }

    async fn generate_block(
        &self,
        pubkey: PublicKey,
        round: u64,
        seed: [u8; 32],
        prev_block_hash: [u8; 32],
        prev_block_timestamp: i64,
    ) -> Block {
        // TODO: fetch mempool transactions

        // Delay next iteration execution so we avoid consensus-split situation.
        // NB: This should be moved to Block Generator when mempool is integrated.
        tokio::time::sleep(Duration::from_millis(config::CONSENSUS_DELAY_MS)).await;

        // TODO: execute state transition

        let blk_header = commons::Header {
            version: 0,
            height: round,
            timestamp: self.get_timestamp(prev_block_timestamp) as i64,
            gas_limit: 0,
            prev_block_hash,
            seed,
            generator_bls_pubkey: pubkey.to_bls_pk().to_bytes(),
            state_hash: [0; 32],
            hash: [0; 32],
        };

        Block::new(blk_header, vec![])
    }

    fn get_timestamp(&self, _prev_block_timestamp: i64) -> u64 {
        // TODO: use config.MaxBlockTime
        if let Ok(n) = SystemTime::now().duration_since(UNIX_EPOCH) {
            return n.as_secs();
        }
        0
    }
}