use std::{cell::Cell, rc::Rc};

use super::{free, NodeKey};

/// A key uniquely identifying a handle within a computational graph.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct AnchorKey {
    pub(super) node_key: NodeKey,
}

impl AnchorKey {
    pub(super) fn new(node_key: NodeKey) -> Self {
        Self { node_key }
    }
}

/// The handle of an anchor from a single-threaded computation graph.
#[derive(Debug)]
pub struct AnchorHandle {
    node_key: NodeKey,
    still_alive: Rc<Cell<bool>>,
}

impl AnchorHandle {
    pub(super) fn new(node_key: NodeKey, still_alive: Rc<Cell<bool>>) -> Self {
        Self {
            node_key,
            still_alive,
        }
    }
}

impl Clone for AnchorHandle {
    fn clone(&self) -> Self {
        if self.still_alive.get() {
            let count = &unsafe { self.node_key.ptr.lookup_unchecked() }
                .ptrs
                .handle_count;
            count.set(count.get() + 1);
        }
        AnchorHandle {
            node_key: self.node_key,
            still_alive: Rc::clone(&self.still_alive),
        }
    }
}

impl Drop for AnchorHandle {
    fn drop(&mut self) {
        if self.still_alive.get() {
            let count = &unsafe { self.node_key.ptr.lookup_unchecked() }
                .ptrs
                .handle_count;
            let new_count = count.get() - 1;
            count.set(new_count);
            if new_count == 0 {
                unsafe { free(self.node_key.ptr) };
            }
        }
    }
}

impl crate::core::AnchorHandle for AnchorHandle {
    type AnchorKey = AnchorKey;

    fn key(&self) -> Self::AnchorKey {
        AnchorKey::new(self.node_key)
    }
}
