use crate::expert::{Poll, UpdateContext};

use super::{
    Anchor, DirtyHandle, Engine, EngineContext, GraphGuard, NodeGuard, ObservedState, RecalcState,
};

pub(super) struct EngineContextMut<'eng, 'gg> {
    engine: &'eng Engine,
    graph: GraphGuard<'gg>,
    node: NodeGuard<'gg>,
    pending_on_anchor_get: bool,
}

impl<'eng, 'gg> EngineContextMut<'eng, 'gg> {
    pub(super) fn new(engine: &'eng Engine, graph: GraphGuard<'gg>, node: NodeGuard<'gg>) -> Self {
        Self {
            engine,
            graph,
            node,
            pending_on_anchor_get: false,
        }
    }

    pub(super) fn pending_on_anchor_get(&self) -> bool {
        self.pending_on_anchor_get
    }
}

impl<'eng, 'gg> UpdateContext for EngineContextMut<'eng, 'gg> {
    type Engine = Engine;

    fn get<'out, 'slf, O>(&'slf self, anchor: &Anchor<O>) -> &'out O
    where
        'slf: 'out,
        O: 'static,
    {
        self.engine.with(|graph| {
            let node = graph.get(anchor.token()).unwrap();
            if super::graph::recalc_state(node) != RecalcState::Ready {
                panic!("attempted to get node that was not previously requested")
            }

            let unsafe_borrow = unsafe { node.anchor.as_ptr().as_ref().unwrap() };
            let output: &O = unsafe_borrow
                .as_ref()
                .unwrap()
                .output(&mut EngineContext::new(self.engine))
                .downcast_ref()
                .unwrap();
            output
        })
    }

    fn request<'out, O>(&mut self, anchor: &Anchor<O>, necessary: bool) -> Poll
    where
        O: 'static,
    {
        let child = self.graph.get(anchor.token()).unwrap();
        let height_already_increased = match super::graph::ensure_height_increases(child, self.node)
        {
            Ok(v) => v,
            Err(()) => {
                panic!("loop detected in anchors!\n");
            }
        };

        let self_is_necessary = Engine::check_observed_raw(self.node) != ObservedState::Unnecessary;

        if super::graph::recalc_state(child) != RecalcState::Ready {
            self.pending_on_anchor_get = true;
            self.graph.queue_recalc(child);
            if necessary && self_is_necessary {
                self.node.add_necessary_child(child);
            }
            Poll::Pending
        } else if !height_already_increased {
            self.pending_on_anchor_get = true;
            Poll::Pending
        } else {
            child.add_clean_parent(self.node);
            if necessary && self_is_necessary {
                self.node.add_necessary_child(child);
            }
            match (child.last_update.get(), self.node.last_ready.get()) {
                (Some(a), Some(b)) if a <= b => Poll::Unchanged,
                _ => Poll::Updated,
            }
        }
    }

    fn unrequest<'out, O>(&mut self, anchor: &Anchor<O>)
    where
        O: 'static,
    {
        let child = self.graph.get(anchor.token()).unwrap();
        self.node.remove_necessary_child(child);
        Engine::update_necessary_children(child);
    }

    fn dirty_handle(&mut self) -> DirtyHandle {
        self.engine.dirty_handle_for_node(self.node.key())
    }
}
