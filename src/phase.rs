// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.
use crate::commons::{RoundUpdate, SelectError};
use crate::consensus::Context;

use crate::frame::Frame;
use crate::selection;
use crate::user::committee::Committee;
use crate::user::provisioners::Provisioners;
use crate::user::sortition;
use crate::{firststep, secondstep};
use tokio::sync::oneshot;
use tracing::trace;

macro_rules! await_phase {
    ($e:expr, $n:ident ( $($args:expr), *), $await:ident) => {
        {
           match $e {
                Phase::Selection(p) => p.$n($($args,)*).await,
                Phase::Reduction1(p) => p.$n($($args,)*).await,
                Phase::Reduction2(p) => p.$n($($args,)*).await,
            }
        }
    };
}

macro_rules! call_phase {
    ($e:expr, $n:ident ( $($args:expr), *)) => {
        {
           match $e {
                Phase::Selection(p) => p.$n($($args,)*),
                Phase::Reduction1(p) => p.$n($($args,)*),
                Phase::Reduction2(p) => p.$n($($args,)*),
            }
        }
    };
}

pub enum Phase {
    Selection(selection::step::Selection),
    Reduction1(firststep::step::Reduction),
    Reduction2(secondstep::step::Reduction),
}

impl Phase {
    pub fn initialize(&mut self, frame: &Frame, round: u64, step: u8) {
        trace!(
            "init phase:{} with frame {:?} at round:{} step:{}",
            self.name(),
            frame,
            round,
            step
        );
        call_phase!(self, initialize(frame))
    }

    pub async fn run(
        &mut self,
        provionsers: &mut Provisioners,
        ctx_recv: &mut oneshot::Receiver<Context>,
        ru: RoundUpdate,
        step: u8,
    ) -> Result<Frame, SelectError> {
        trace!("running phase:{} round:{:?} step:{}", self.name(), ru, step);

        let size = call_phase!(self, get_committee_size());

        // Perform deterministic_sortition to generate committee of size=N.
        // The extracted members are the provisioners eligible to vote on this particular round and step.
        // In the context of Selection phase, the extracted member is the one eligible to generate the candidate block.
        let step_committee = Committee::new(
            ru.pubkey_bls.clone(),
            provionsers,
            sortition::Config(ru.seed, ru.round, step, size),
        );

        // TODO: consider here to execute sortition and pass committee instead of provisioners
        await_phase!(self, run(ctx_recv,  step_committee, ru, step), await)
    }

    fn name(&self) -> String {
        call_phase!(self, name())
    }
}
