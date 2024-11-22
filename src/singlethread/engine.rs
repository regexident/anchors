use std::{cell::RefCell, rc::Rc};

use crate::expert::{AnchorInner, Poll};

use super::{
    Anchor, AnchorHandle, DirtyHandle, EngineContext, EngineContextMut, Generation, GenericAnchor,
    Graph, GraphGuard, Mounter, NodeGuard, NodeKey, ObservedState, RecalcState, DEFAULT_MOUNTER,
};

/// The main execution engine of Single-thread.
pub struct Engine {
    // TODO store Nodes on heap directly?? maybe try for Rc<RefCell<SlotMap>> now
    graph: Rc<Graph>,
    pub(super) dirty_marks: Rc<RefCell<Vec<NodeKey>>>,

    // tracks the current stabilization generation; incremented on every stabilize
    generation: Generation,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::expert::Engine for Engine {
    type AnchorHandle = AnchorHandle;
    type DirtyHandle = DirtyHandle;

    fn mount<I: 'static + AnchorInner<Self>>(inner: I) -> Anchor<I::Output> {
        DEFAULT_MOUNTER.with(|default_mounter| {
            let mut borrow1 = default_mounter.borrow_mut();
            let this = borrow1
                .as_mut()
                .expect("no engine was initialized. did you call `Engine::new()`?");
            let debug_info = inner.debug_info();
            let handle = this.graph.insert(Box::new(inner), debug_info);
            Anchor::new_from_expert(handle)
        })
    }
}

impl Engine {
    /// Creates a new Engine with maximum height 256.
    pub fn new() -> Self {
        Self::new_with_max_height(256)
    }

    /// Creates a new Engine with a custom maximum height.
    pub fn new_with_max_height(max_height: usize) -> Self {
        let graph = Rc::new(Graph::new(max_height));
        let mounter = Mounter {
            graph: graph.clone(),
        };
        DEFAULT_MOUNTER.with(|v| *v.borrow_mut() = Some(mounter));
        Self {
            graph,
            dirty_marks: Default::default(),
            generation: Generation::new(),
        }
    }

    pub fn with<F: for<'any> FnOnce(GraphGuard<'any>) -> R, R>(&self, f: F) -> R {
        self.graph.with(f)
    }

    /// Marks an Anchor as observed. All observed nodes will always be brought up-to-date
    /// when *any* Anchor in the graph is retrieved. If you get an output value fairly
    /// often, it's best to mark it as Observed so that Anchors can calculate its
    /// dependencies faster.
    pub fn mark_observed<O>(&mut self, anchor: &Anchor<O>)
    where
        O: 'static,
    {
        self.with(|graph| {
            let node = graph.get(anchor.token()).unwrap();
            node.observed.set(true);
            if super::graph::recalc_state(node) != RecalcState::Ready {
                graph.queue_recalc(node);
            }
        })
    }

    /// Marks an Anchor as unobserved. If the `anchor` has parents that are necessary
    /// because `anchor` was previously observed, those parents will be unmarked as
    /// necessary.
    pub fn mark_unobserved<O>(&mut self, anchor: &Anchor<O>)
    where
        O: 'static,
    {
        self.with(|graph| {
            let node = graph.get(anchor.token()).unwrap();
            node.observed.set(false);
            Self::update_necessary_children(node);
        })
    }

    pub(super) fn update_necessary_children(node: NodeGuard<'_>) {
        if Self::check_observed_raw(node) != ObservedState::Unnecessary {
            // we have another parent still observed, so skip this
            return;
        }
        for child in node.drain_necessary_children() {
            // TODO remove from calculation queue if necessary?
            Self::update_necessary_children(child);
        }
    }

    /// Retrieves the value of an Anchor, recalculating dependencies as necessary to get the
    /// latest value.
    pub fn get<O>(&mut self, anchor: &Anchor<O>) -> O
    where
        O: 'static + Clone,
    {
        // stabilize once before, since the stabilization process may mark our requested node
        // as dirty
        self.stabilize();
        self.with(|graph| {
            let anchor_node = graph.get(anchor.token()).unwrap();
            if super::graph::recalc_state(anchor_node) != RecalcState::Ready {
                graph.queue_recalc(anchor_node);
                // stabilize again, to make sure our target node that is now in the queue is up-to-date
                // use stabilize0 because no dirty marks have occurred since last stabilization, and we want
                // to make sure we don't unnecessarily increment generation number
                self.stabilize0();
            }
            let target_anchor = &graph.get(anchor.token()).unwrap().anchor;
            let borrow = target_anchor.borrow();
            borrow
                .as_ref()
                .unwrap()
                .output(&mut EngineContext::new(self))
                .downcast_ref::<O>()
                .unwrap()
                .clone()
        })
    }

    pub(crate) fn update_dirty_marks(&mut self) {
        self.with(|graph| {
            let dirty_marks = std::mem::take(&mut *self.dirty_marks.borrow_mut());
            for dirty in dirty_marks {
                let node = graph.get(dirty).unwrap();
                super::mark_dirty(graph, node, false);
            }
        })
    }

    /// Ensure any Observed nodes are up-to-date, recalculating dependencies as necessary. You
    /// should rarely need to call this yourself; `Engine::get` calls it automatically.
    pub fn stabilize(&mut self) {
        self.update_dirty_marks();
        self.generation.increment();
        self.stabilize0();
    }

    /// internal function for stabilization. does not update dirty marks or increment the stabilization number
    fn stabilize0(&self) {
        self.with(|graph| {
            while let Some((height, node)) = graph.recalc_pop_next() {
                let calculation_complete = if super::graph::height(node) == height {
                    // TODO with new graph we can automatically relocate nodes if their height changes
                    // this nodes height is current, so we can recalculate
                    self.recalculate(graph, node)
                } else {
                    // skip calculation, redo at correct height
                    false
                };

                if !calculation_complete {
                    graph.queue_recalc(node);
                }
            }
        })
    }

    /// returns false if calculation is still pending
    fn recalculate<'a>(&self, graph: GraphGuard<'a>, node: NodeGuard<'a>) -> bool {
        let this_anchor = &node.anchor;
        let mut ecx = EngineContextMut::new(self, graph, node);
        let poll_result = this_anchor
            .borrow_mut()
            .as_mut()
            .unwrap()
            .poll_updated(&mut ecx);
        match poll_result {
            Poll::Pending => {
                if ecx.pending_on_anchor_get() {
                    // looks like we requested an anchor that isn't yet calculated, so we
                    // reinsert into the graph directly; our height either was higher than this
                    // requested anchor's already, or it was updated so it's higher now.
                    false
                } else {
                    // in the future, this means we polled on some non-anchors future. since
                    // that isn't supported for now, this just means something went wrong
                    panic!("poll_updated return pending without requesting another anchor");
                }
            }
            Poll::Updated => {
                // make sure all parents are marked as dirty, and observed parents are recalculated
                super::mark_dirty(graph, node, true);
                node.last_update.set(Some(self.generation));
                node.last_ready.set(Some(self.generation));
                true
            }
            Poll::Unchanged => {
                node.last_ready.set(Some(self.generation));
                true
            }
        }
    }

    /// Returns a debug string containing the current state of the recomputation graph.
    pub fn debug_state(&self) -> String {
        let debug = "".to_string();
        // for (node_id, _) in nodes.iter() {
        //     let node = self.graph.get(node_id).unwrap();
        //     let necessary = if self.graph.is_necessary(node_id) {
        //         "necessary"
        //     } else {
        //         "   --    "
        //     };
        //     let observed = if Self::check_observed_raw(node) == ObservedState::Observed {
        //         "observed"
        //     } else {
        //         "   --   "
        //     };
        //     let state = match self.to_recalculate.borrow_mut().state(node_id) {
        //         RecalcState::NeedsRecalc => "NeedsRecalc  ",
        //         RecalcState::PendingRecalc => "PendingRecalc",
        //         RecalcState::Ready => "Ready        ",
        //     };
        //     debug += &format!(
        //         "{:>80}  {}  {}  {}\n",
        //         node.debug_info.get().to_string(),
        //         necessary,
        //         observed,
        //         state
        //     );
        // }
        #[allow(clippy::let_and_return)]
        debug
    }

    pub fn check_observed<T>(&self, anchor: &Anchor<T>) -> ObservedState {
        self.with(|graph| {
            let node = graph.get(anchor.token()).unwrap();
            Self::check_observed_raw(node)
        })
    }

    /// Returns whether an Anchor is Observed, Necessary, or Unnecessary.
    pub fn check_observed_raw(node: NodeGuard<'_>) -> ObservedState {
        if node.observed.get() {
            return ObservedState::Observed;
        }

        if node.necessary_count.get() > 0 {
            ObservedState::Necessary
        } else {
            ObservedState::Unnecessary
        }
    }
}
