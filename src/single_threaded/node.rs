use std::cell::{Cell, RefCell};

use crate::arena;

use super::{generation::Generation, node_ptrs::NodePtrs, AnchorDebugInfo, GenericAnchor};

pub(super) struct Node {
    pub observed: Cell<bool>,

    /// Bool used during height incrementing to check for loops
    pub visited: Cell<bool>,

    /// Number of nodes that list `self` as a necessary child.
    pub necessary_count: Cell<usize>,

    pub token: u32,

    pub(super) debug_info: Cell<AnchorDebugInfo>,

    /// Tracks when this `Node`` was last polled as `Updated` or `Unchanged`.
    pub(super) last_ready: Cell<Option<Generation>>,
    /// Tracks when this `Node` was` last polled as `Updated`.
    pub(super) last_update: Cell<Option<Generation>>,

    /// `Some(_)`` if this node is still active, `None`` otherwise
    pub(super) anchor: RefCell<Option<Box<dyn GenericAnchor>>>,

    pub ptrs: NodePtrs,
}

pub(super) type NodePtr = arena::NodePtr<Node>;
