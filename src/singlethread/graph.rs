use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use crate::arena;

use super::{
    node::Node, AnchorDebugInfo, AnchorHandle, GenericAnchor, GraphGuard, NodeGuard, NodeKey,
    NodePtr, NodePtrs,
};

#[derive(Copy, Clone, Default, Eq, PartialEq, Hash, Debug)]
pub(super) enum RecalcState {
    #[default]
    Needed,
    Pending,
    Ready,
}

thread_local! {
    static NEXT_TOKEN: Cell<u32> = const { Cell::new(0) };
}

pub(super) struct Graph {
    pub(super) nodes: arena::Graph<Node>,
    token: u32,

    pub(super) still_alive: Rc<Cell<bool>>,

    /// height -> first node in that height's queue
    pub(super) recalc_queues: RefCell<Vec<Option<NodePtr>>>,
    pub(super) recalc_min_height: Cell<usize>,
    pub(super) recalc_max_height: Cell<usize>,

    /// pointer to head of linked list of free nodes
    pub(super) free_head: Box<Cell<Option<NodePtr>>>,
}

impl Graph {
    pub fn new(max_height: usize) -> Self {
        Self {
            nodes: arena::Graph::new(),
            token: NEXT_TOKEN.with(|token| {
                let n = token.get();
                token.set(n + 1);
                n
            }),
            recalc_queues: RefCell::new(vec![None; max_height]),
            recalc_min_height: Cell::new(max_height),
            recalc_max_height: Cell::new(0),
            still_alive: Rc::new(Cell::new(true)),
            free_head: Box::new(Cell::new(None)),
        }
    }

    pub(super) fn accepts_key(&self, node_key: NodeKey) -> bool {
        node_key.token == self.token
    }

    pub fn with<F: for<'any> FnOnce(GraphGuard<'any>) -> R, R>(&self, f: F) -> R {
        let nodes = unsafe { self.nodes.with_unchecked() };
        f(GraphGuard::new(nodes, self))
    }

    #[cfg(test)]
    pub(super) fn insert_testing(&self) -> AnchorHandle {
        use crate::core::Constant;

        self.insert(
            Box::new(Constant::new_raw_testing(123)),
            AnchorDebugInfo {
                location: None,
                type_info: "testing dummy anchor",
            },
        )
    }

    pub(super) fn insert(
        &self,
        anchor: Box<dyn GenericAnchor>,
        debug_info: AnchorDebugInfo,
    ) -> AnchorHandle {
        self.nodes.with(|nodes| {
            let ptr = if let Some(free_head) = self.free_head.get() {
                let node = unsafe { nodes.lookup_ptr(free_head) };
                self.free_head.set(node.ptrs.next.get());
                if let Some(next_ptr) = node.ptrs.next.get() {
                    let next_node = unsafe { nodes.lookup_ptr(next_ptr) };
                    next_node.ptrs.prev.set(None);
                }
                node.observed.set(false);
                node.visited.set(false);
                node.necessary_count.set(0);
                node.ptrs.clean_parent0.set(None);
                node.ptrs.clean_parents.replace(vec![]);
                node.ptrs.recalc_state.set(RecalcState::Needed);
                node.ptrs.necessary_children.replace(vec![]);
                node.ptrs.height.set(0);
                node.ptrs.handle_count.set(1);
                node.ptrs.prev.set(None);
                node.ptrs.next.set(None);
                node.debug_info.set(debug_info);
                node.last_ready.set(None);
                node.last_update.set(None);
                node.anchor.replace(Some(anchor));
                node
            } else {
                let node = Node {
                    observed: Cell::new(false),
                    visited: Cell::new(false),
                    necessary_count: Cell::new(0),
                    token: self.token,
                    ptrs: NodePtrs {
                        clean_parent0: Cell::new(None),
                        clean_parents: RefCell::new(vec![]),
                        graph: self,
                        next: Cell::new(None),
                        prev: Cell::new(None),
                        recalc_state: Cell::new(RecalcState::Needed),
                        necessary_children: RefCell::new(vec![]),
                        height: Cell::new(0),
                        handle_count: Cell::new(1),
                    },
                    debug_info: Cell::new(debug_info),
                    last_ready: Cell::new(None),
                    last_update: Cell::new(None),
                    anchor: RefCell::new(Some(anchor)),
                };
                nodes.insert(node)
            };
            let num = NodeKey::new(unsafe { ptr.make_ptr() }, self.token);
            AnchorHandle::new(num, self.still_alive.clone())
        })
    }
}

impl Drop for Graph {
    fn drop(&mut self) {
        self.still_alive.set(false);
    }
}

#[allow(clippy::result_unit_err)] // FIXME
pub(super) fn ensure_height_increases<'a>(
    child: NodeGuard<'a>,
    parent: NodeGuard<'a>,
) -> Result<bool, ()> {
    if height(child) < height(parent) {
        return Ok(true);
    }
    child.visited.set(true);
    let res = set_min_height(parent, height(child) + 1);
    child.visited.set(false);
    res.map(|()| false)
}

#[allow(clippy::result_unit_err)] // FIXME
pub(super) fn set_min_height(node: NodeGuard<'_>, min_height: usize) -> Result<(), ()> {
    if node.visited.get() {
        return Err(());
    }

    node.visited.set(true);

    if height(node) < min_height {
        node.ptrs.height.set(min_height);
        let mut did_err = false;
        for parent in node.clean_parents() {
            if let Err(_loop_ids) = set_min_height(parent, min_height + 1) {
                did_err = true;
            }
        }
        if did_err {
            return Err(());
        }
    }

    node.visited.set(false);

    Ok(())
}

pub(super) unsafe fn free(ptr: NodePtr) {
    let guard = NodeGuard(ptr.lookup_unchecked());
    let _ = guard.drain_necessary_children();
    let _ = guard.drain_clean_parents();
    let graph = &*guard.ptrs.graph;
    dequeue_calc(graph, guard);
    // TODO clear out this node with default empty data
    // TODO add node to chain of free nodes
    let free_head = &graph.free_head;
    let old_free = free_head.get();

    if let Some(old_free) = old_free {
        guard.0.lookup_ptr(old_free).ptrs.prev.set(Some(ptr));
    }

    guard.ptrs.next.set(old_free);
    free_head.set(Some(ptr));

    // "SAFETY": this may cause other nodes to be dropped, so do with care
    *guard.anchor.borrow_mut() = None;
}

fn dequeue_calc(graph: &Graph, node: NodeGuard<'_>) {
    if node.ptrs.recalc_state.get() != RecalcState::Pending {
        return;
    }

    if let Some(prev) = node.ptrs.prev.get() {
        unsafe { prev.lookup_unchecked() }
            .ptrs
            .next
            .set(node.ptrs.next.get());
    } else {
        // node was first in queue, need to set queue head to next
        let mut recalc_queues = graph.recalc_queues.borrow_mut();
        let height = node.ptrs.height.get();
        let next = node.ptrs.next.get();
        assert_eq!(
            recalc_queues[height].map(|ptr| unsafe { ptr.lookup_unchecked() }),
            Some(node.0)
        );
        recalc_queues[height] = next;
    }

    if let Some(next) = node.ptrs.next.get() {
        unsafe { next.lookup_unchecked() }
            .ptrs
            .next
            .set(node.ptrs.prev.get());
    }

    node.ptrs.prev.set(None);
    node.ptrs.next.set(None);
}

pub(super) fn height(node: NodeGuard<'_>) -> usize {
    node.ptrs.height.get()
}

pub(super) fn needs_recalc(node: NodeGuard<'_>) {
    if node.ptrs.recalc_state.get() != RecalcState::Ready {
        // already in recalc queue, or already pending recalc
        return;
    }

    node.ptrs.recalc_state.set(RecalcState::Needed);
}

pub(super) fn recalc_state(node: NodeGuard<'_>) -> RecalcState {
    node.ptrs.recalc_state.get()
}

#[cfg(test)]
mod tests;
