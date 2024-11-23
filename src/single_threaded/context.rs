use crate::{core::OutputContext, single_threaded::*};

use super::Engine;

pub(super) struct EngineContext<'eng> {
    engine: &'eng Engine,
}

impl<'eng> EngineContext<'eng> {
    pub(super) fn new(engine: &'eng Engine) -> Self {
        Self { engine }
    }
}

impl<'eng> OutputContext<'eng> for EngineContext<'eng> {
    type Engine = Engine;

    fn get<'out, O>(&self, anchor: &Anchor<O>) -> &'out O
    where
        'eng: 'out,
        O: 'static,
    {
        self.engine.with(|graph| {
            let node = graph.get(anchor.key().node_key).unwrap();
            if graph::recalc_state(node) != RecalcState::Ready {
                panic!("attempted to get node that was not previously requested")
            }
            let unsafe_borrow = unsafe { node.anchor.as_ptr().as_ref().unwrap() };
            let output: &O = unsafe_borrow
                .as_ref()
                .unwrap()
                .output(&mut EngineContext {
                    engine: self.engine,
                })
                .downcast_ref()
                .unwrap();
            output
        })
    }
}
