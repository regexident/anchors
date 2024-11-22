use std::cell::{Cell, RefCell};

use super::{Graph, NodePtr, RecalcState};

pub struct NodePtrs {
    /// first parent, remaining parents. unsorted, duplicates may exist
    pub(super) clean_parent0: Cell<Option<NodePtr>>,
    pub(super) clean_parents: RefCell<Vec<NodePtr>>,

    pub(super) graph: *const Graph,

    /// Next node in either recalc linked list for this height, or if node is in the free list, the free linked list.
    /// If this is the last node, None.
    pub(super) next: Cell<Option<NodePtr>>,
    /// Prev node in either recalc linked list for this height, or if node is in the free list, the free linked list.
    /// If this is the head node, None.
    pub(super) prev: Cell<Option<NodePtr>>,
    pub(super) recalc_state: Cell<RecalcState>,

    /// sorted in pointer order
    pub(super) necessary_children: RefCell<Vec<NodePtr>>,

    pub(super) height: Cell<usize>,

    pub(super) handle_count: Cell<usize>,
}
