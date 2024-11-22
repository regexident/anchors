use crate::arena;

use super::{node::Node, NodeKey, RefCellVecIterator};

#[derive(Copy, Clone, Debug)]
pub struct NodeGuard<'a>(pub(super) arena::NodeGuard<'a, Node>);

impl<'a> std::ops::Deref for NodeGuard<'a> {
    type Target = Node;

    fn deref(&self) -> &Node {
        self.0.node()
    }
}

impl<'a> PartialEq for NodeGuard<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<'a> NodeGuard<'a> {
    pub fn key(self) -> NodeKey {
        NodeKey::new(unsafe { self.0.make_ptr() }, self.token)
    }

    pub fn add_clean_parent(self, parent: NodeGuard<'a>) {
        if self.ptrs.clean_parent0.get().is_none() {
            self.ptrs
                .clean_parent0
                .set(Some(unsafe { parent.0.make_ptr() }))
        } else {
            self.ptrs
                .clean_parents
                .borrow_mut()
                .push(unsafe { parent.0.make_ptr() })
        }
    }

    pub fn clean_parents(self) -> impl Iterator<Item = NodeGuard<'a>> {
        RefCellVecIterator::new(
            self.0.node().ptrs.clean_parents.borrow_mut(),
            0,
            self.ptrs.clean_parent0.get(),
            false,
        )
    }

    pub fn drain_clean_parents(self) -> impl Iterator<Item = NodeGuard<'a>> {
        RefCellVecIterator::new(
            self.0.node().ptrs.clean_parents.borrow_mut(),
            0,
            self.ptrs.clean_parent0.take(),
            true,
        )
    }

    pub fn add_necessary_child(self, child: NodeGuard<'a>) {
        let mut necessary_children = self.ptrs.necessary_children.borrow_mut();
        let child_ptr = unsafe { child.0.make_ptr() };
        if let Err(i) = necessary_children.binary_search(&child_ptr) {
            necessary_children.insert(i, child_ptr);
            child.necessary_count.set(child.necessary_count.get() + 1)
        }
    }

    pub fn remove_necessary_child(self, child: NodeGuard<'a>) {
        let mut necessary_children = self.ptrs.necessary_children.borrow_mut();
        let child_ptr = unsafe { child.0.make_ptr() };
        if let Ok(i) = necessary_children.binary_search(&child_ptr) {
            necessary_children.remove(i);
            child.necessary_count.set(child.necessary_count.get() - 1)
        }
    }

    pub fn necessary_children(self) -> impl Iterator<Item = NodeGuard<'a>> {
        RefCellVecIterator::new(
            self.0.node().ptrs.necessary_children.borrow_mut(),
            0,
            None,
            false,
        )
    }

    pub fn drain_necessary_children(self) -> impl Iterator<Item = NodeGuard<'a>> {
        let necessary_children = self.0.node().ptrs.necessary_children.borrow_mut();
        for child in &*necessary_children {
            let count = &unsafe { self.0.lookup_ptr(*child) }.necessary_count;
            count.set(count.get() - 1);
        }
        RefCellVecIterator::new(necessary_children, 0, None, true)
    }
}
