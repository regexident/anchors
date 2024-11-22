use std::{cell::Cell, rc::Rc};

use super::{free, NodeKey};

/// Singlethread's implementation of Anchors' `AnchorHandle`, the engine-specific handle that sits inside an `Anchor`.
#[derive(Debug)]
pub struct AnchorHandle {
    key: NodeKey,
    still_alive: Rc<Cell<bool>>,
}

impl AnchorHandle {
    pub fn new(key: NodeKey, still_alive: Rc<Cell<bool>>) -> Self {
        Self { key, still_alive }
    }

    #[allow(dead_code)]
    pub(super) fn key(&self) -> NodeKey {
        self.key
    }
}

impl Clone for AnchorHandle {
    fn clone(&self) -> Self {
        if self.still_alive.get() {
            let count = &unsafe { self.key.ptr.lookup_unchecked() }.ptrs.handle_count;
            count.set(count.get() + 1);
        }
        AnchorHandle {
            key: self.key,
            still_alive: self.still_alive.clone(),
        }
    }
}

impl Drop for AnchorHandle {
    fn drop(&mut self) {
        if self.still_alive.get() {
            let count = &unsafe { self.key.ptr.lookup_unchecked() }.ptrs.handle_count;
            let new_count = count.get() - 1;
            count.set(new_count);
            if new_count == 0 {
                unsafe { free(self.key.ptr) };
            }
        }
    }
}

impl crate::expert::AnchorHandle for AnchorHandle {
    type Token = NodeKey;

    fn token(&self) -> NodeKey {
        self.key
    }
}
