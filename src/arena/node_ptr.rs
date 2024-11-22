use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
    marker::PhantomData,
    ptr::NonNull,
};

use super::node_guard::NodeGuard;

pub struct NodePtr<N>(pub(super) NonNull<N>);

impl<N> NodePtr<N> {
    #[allow(dead_code)]
    pub fn ptr_eq(self, other: Self) -> bool {
        std::ptr::eq(self.0.as_ptr(), other.0.as_ptr())
    }

    pub unsafe fn lookup_unchecked<'gg>(&self) -> NodeGuard<'gg, N> {
        NodeGuard {
            node: &*self.0.as_ptr(),
            invariant: PhantomData,
        }
    }
}

impl<N> Clone for NodePtr<N> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<N> Copy for NodePtr<N> {}

impl<N> PartialOrd for NodePtr<N> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<N> Ord for NodePtr<N> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl<N> PartialEq for NodePtr<N> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<N> Eq for NodePtr<N> {}

impl<N> Hash for NodePtr<N> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<N> std::fmt::Debug for NodePtr<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodePtr").finish()
    }
}
