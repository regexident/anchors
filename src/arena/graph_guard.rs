use std::marker::PhantomData;

use super::{graph::Graph, node_guard::NodeGuard, node_ptr::NodePtr};

pub struct GraphGuard<'gg, N> {
    pub(super) graph: &'gg Graph<N>,
    pub(super) invariant: PhantomData<&'gg mut &'gg ()>,
}

impl<'gg, N> GraphGuard<'gg, N> {
    pub fn insert(&self, node: N) -> NodeGuard<'gg, N> {
        let node_ref = self.graph.arena.alloc(node);
        NodeGuard {
            node: node_ref,
            invariant: self.invariant,
        }
    }

    pub unsafe fn lookup_ptr(&self, ptr: NodePtr<N>) -> NodeGuard<'gg, N> {
        NodeGuard {
            node: &*ptr.0.as_ptr(),
            invariant: self.invariant,
        }
    }
}

impl<N> Clone for GraphGuard<'_, N> {
    fn clone(&self) -> Self {
        Self {
            graph: self.graph,
            invariant: self.invariant,
        }
    }
}

impl<N> Copy for GraphGuard<'_, N> {}
