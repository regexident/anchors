use std::{marker::PhantomData, ops::Deref, ptr::NonNull};

use super::node_ptr::NodePtr;

pub struct NodeGuard<'gg, N> {
    pub(super) node: &'gg N,
    pub(super) invariant: PhantomData<&'gg mut &'gg ()>,
}

impl<N> Clone for NodeGuard<'_, N> {
    fn clone(&self) -> Self {
        Self {
            node: self.node,
            invariant: self.invariant,
        }
    }
}

impl<N> Copy for NodeGuard<'_, N> {}

impl<N> PartialEq for NodeGuard<'_, N> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.node, other.node)
    }
}

impl<N> Eq for NodeGuard<'_, N> {}

impl<'gg, N> Deref for NodeGuard<'gg, N> {
    type Target = N;

    fn deref(&self) -> &N {
        self.node
    }
}

impl<N> std::fmt::Debug for NodeGuard<'_, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeGuard").finish()
    }
}

impl<'gg, N> NodeGuard<'gg, N> {
    pub unsafe fn make_ptr(&self) -> NodePtr<N> {
        // ideally would not have to cast to *mut N, but will need to until we get NonNullConst
        NodePtr(NonNull::new_unchecked(self.node as *const N as *mut N))
    }

    pub unsafe fn lookup_ptr(&self, ptr: NodePtr<N>) -> NodeGuard<'gg, N> {
        NodeGuard {
            node: &*ptr.0.as_ptr(),
            invariant: self.invariant,
        }
    }

    pub fn node(&self) -> &'gg N {
        self.node
    }
}
