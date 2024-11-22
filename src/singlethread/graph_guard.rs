use crate::arena;

use super::{Graph, Node, NodeGuard, NodeKey, RecalcState};

#[derive(Copy, Clone)]
pub(super) struct GraphGuard<'gg> {
    nodes: arena::GraphGuard<'gg, Node>,
    graph: &'gg Graph,
}

impl<'gg> GraphGuard<'gg> {
    pub(super) fn new(nodes: arena::GraphGuard<'gg, Node>, graph: &'gg Graph) -> Self {
        Self { nodes, graph }
    }

    pub(super) fn get(&self, key: NodeKey) -> Option<NodeGuard<'gg>> {
        if !self.graph.accepts_key(key) {
            return None;
        }

        Some(NodeGuard(unsafe { self.nodes.lookup_ptr(key.ptr) }))
    }

    #[cfg(test)]
    pub(super) fn insert_testing_guard(&self) -> NodeGuard<'gg> {
        use crate::core::AnchorHandle as _;

        let handle = self.graph.insert_testing();
        let guard = self.get(handle.key().node_key).unwrap();
        std::mem::forget(handle);
        guard
    }

    pub(super) fn recalc_pop_next(&self) -> Option<(usize, NodeGuard<'gg>)> {
        let mut recalc_queues = self.graph.recalc_queues.borrow_mut();
        while self.graph.recalc_min_height.get() <= self.graph.recalc_max_height.get() {
            if let Some(ptr) = recalc_queues[self.graph.recalc_min_height.get()] {
                let node = unsafe { self.nodes.lookup_ptr(ptr) };
                recalc_queues[self.graph.recalc_min_height.get()] = node.ptrs.next.get();
                if let Some(next_in_queue_ptr) = node.ptrs.next.get() {
                    unsafe { self.nodes.lookup_ptr(next_in_queue_ptr) }
                        .ptrs
                        .prev
                        .set(None);
                }
                node.ptrs.prev.set(None);
                node.ptrs.next.set(None);
                node.ptrs.recalc_state.set(RecalcState::Ready);
                return Some((self.graph.recalc_min_height.get(), NodeGuard(node)));
            } else {
                self.graph
                    .recalc_min_height
                    .set(self.graph.recalc_min_height.get() + 1);
            }
        }
        self.graph.recalc_max_height.set(0);
        None
    }

    pub(super) fn queue_recalc(&self, node: NodeGuard<'gg>) {
        if node.ptrs.recalc_state.get() == RecalcState::Pending {
            // already in recalc queue
            return;
        }
        node.ptrs.recalc_state.set(RecalcState::Pending);
        let node_height = super::height(node);
        let mut recalc_queues = self.graph.recalc_queues.borrow_mut();
        if node_height >= recalc_queues.len() {
            panic!("too large height error");
        }
        if let Some(old) = recalc_queues[node_height] {
            unsafe { self.nodes.lookup_ptr(old) }
                .ptrs
                .prev
                .set(Some(unsafe { node.0.make_ptr() }));
            node.ptrs.next.set(Some(old));
        } else {
            if self.graph.recalc_min_height.get() > node_height {
                self.graph.recalc_min_height.set(node_height);
            }
            if self.graph.recalc_max_height.get() < node_height {
                self.graph.recalc_max_height.set(node_height);
            }
        }
        recalc_queues[node_height] = Some(unsafe { node.0.make_ptr() });
    }
}
