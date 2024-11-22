//! Anchors' default execution engine.
//!
//! It's a single threaded engine capable of both Adapton-style pull updates
//! and — if `mark_observed` and `mark_unobserved` are used, Incremental-style push updates.
//!
//! As of September 2020, execution overhead per-node sits at around 100ns on this author's MacBook Air,
//! likely somewhat more if single node has a significant number of parents or children.
//! Hopefully this will significantly improve over the coming months.

use std::{cell::RefCell, rc::Rc};

pub use crate::expert::MultiAnchor;

mod anchor;
mod anchor_handle;
mod context;
mod context_mut;
mod engine;
mod generation;
mod graph;
mod graph_guard;
mod node;
mod node_guard;
mod node_iterator;
mod node_key;
mod node_ptrs;

pub use self::{
    anchor_handle::*, engine::*, graph::*, graph_guard::*, node::*, node_guard::*, node_key::*,
    node_ptrs::*,
};

use self::{anchor::*, context::*, context_mut::*, generation::*, node_iterator::*};

/// The main struct of the Anchors library. Represents a single value on the `singlethread` recomputation graph.
///
/// You should basically never need to create these with `Anchor::new_from_expert`; instead call functions like `Var::new` and `MultiAnchor::map`
/// to create them.
pub type Anchor<T> = crate::expert::Anchor<T, Engine>;

/// An Anchor input that can be mutated by calling a setter function from outside of the Anchors recomputation graph.
pub type Var<T> = crate::expert::Var<T, Engine>;

thread_local! {
    static DEFAULT_MOUNTER: RefCell<Option<Mounter>> = const { RefCell::new(None) };
}

/// Indicates whether the node is a part of some observed calculation.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ObservedState {
    /// The node has been marked as observed directly via `mark_observed`.
    Observed,

    /// The node is not marked as observed directly.
    /// However, the node has some descendent that is Observed, and this node has
    /// been recalculated since that descendent become Observed.
    Necessary,

    /// The node is not marked as observed directly.
    /// Additionally, this node either has no Observed descendent, or the chain linking
    /// this node to that Observed descendent has not been recalculated since that
    /// descendent become observed.
    Unnecessary,
}

struct Mounter {
    graph: Rc<Graph>,
}

// skip_self = true indicates output has *definitely* changed, but node has been recalculated
// skip_self = false indicates node has not yet been recalculated
fn mark_dirty<'a>(graph: GraphGuard<'a>, node: NodeGuard<'a>, skip_self: bool) {
    if skip_self {
        let parents = node.drain_clean_parents();
        for parent in parents {
            // TODO still calling dirty twice on observed relationships
            parent
                .anchor
                .borrow_mut()
                .as_mut()
                .unwrap()
                .dirty(&node.key());
            mark_dirty0(graph, parent);
        }
    } else {
        mark_dirty0(graph, node);
    }
}

fn mark_dirty0<'a>(graph: GraphGuard<'a>, next: NodeGuard<'a>) {
    let id = next.key();
    if Engine::check_observed_raw(next) != ObservedState::Unnecessary {
        graph.queue_recalc(next);
    } else if graph::recalc_state(next) == RecalcState::Ready {
        graph::needs_recalc(next);
        let parents = next.drain_clean_parents();
        for parent in parents {
            if let Some(v) = parent.anchor.borrow_mut().as_mut() {
                v.dirty(&id);
                mark_dirty0(graph, parent);
            }
        }
    }
}

/// Single-threaded implementation of Anchors' `DirtyHandle`, which allows a node with non-Anchors inputs to manually mark itself as dirty.
#[derive(Clone, Debug)]
pub struct DirtyHandle {
    key: NodeKey,
    dirty_marks: Rc<RefCell<Vec<NodeKey>>>,
}

impl crate::expert::DirtyHandle for DirtyHandle {
    fn mark_dirty(&self) {
        self.dirty_marks.borrow_mut().push(self.key);
    }
}

#[cfg(test)]
mod tests;
