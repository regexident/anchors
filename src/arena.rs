// The contents of this module were originally in a separate crate
// (by the same original author, Robert Lord):
//
// https://crates.io/crates/arena-graph
//
// But since both projects' repositories were archived on 2023-11-22,
// I (Vincent Esche) decided to merge them in an effort to revive them:
// The `arena-graph` crate doesn't seem to have much use on its own,
// consists of basically a single file, and it's easier to maintain them this way.
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::NonNull;

use typed_arena::Arena;

pub struct Graph<N> {
    graph: Arena<N>,
}

impl<N> Graph<N> {
    pub fn new() -> Self {
        Graph {
            graph: Arena::new(),
        }
    }

    pub fn with<F: for<'any> FnOnce(GraphGuard<'any, N>) -> R, R>(&self, func: F) -> R {
        func(GraphGuard {
            inside: self,
            invariant: PhantomData,
        })
    }

    pub unsafe fn with_unchecked<'gg, 'slf>(&'slf self) -> GraphGuard<'gg, N>
    where
        'slf: 'gg,
    {
        GraphGuard {
            inside: self,
            invariant: PhantomData,
        }
    }
}

impl<N> Default for Graph<N> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct GraphGuard<'gg, N> {
    inside: &'gg Graph<N>,
    invariant: PhantomData<&'gg mut &'gg ()>,
}

impl<'gg, N> GraphGuard<'gg, N> {
    pub fn insert(&self, node: N) -> NodeGuard<'gg, N> {
        let node_ref = self.inside.graph.alloc(node);
        NodeGuard {
            inside: node_ref,
            invariant: self.invariant,
        }
    }

    pub unsafe fn lookup_ptr(&self, NodePtr(ptr): NodePtr<N>) -> NodeGuard<'gg, N> {
        NodeGuard {
            inside: &*ptr.as_ptr(),
            invariant: self.invariant,
        }
    }
}

pub struct NodeGuard<'gg, N> {
    inside: &'gg N,
    invariant: PhantomData<&'gg mut &'gg ()>,
}

impl<N> Clone for GraphGuard<'_, N> {
    fn clone(&self) -> Self {
        Self {
            inside: self.inside,
            invariant: self.invariant,
        }
    }
}

impl<N> Copy for GraphGuard<'_, N> {}

impl<N> Clone for NodeGuard<'_, N> {
    fn clone(&self) -> Self {
        Self {
            inside: self.inside,
            invariant: self.invariant,
        }
    }
}

impl<N> Copy for NodeGuard<'_, N> {}

impl<N> PartialEq for NodeGuard<'_, N> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.inside, other.inside)
    }
}

impl<N> Eq for NodeGuard<'_, N> {}

impl<'gg, N> Deref for NodeGuard<'gg, N> {
    type Target = N;
    fn deref(&self) -> &N {
        self.inside
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
        NodePtr(NonNull::new_unchecked(self.inside as *const N as *mut N))
    }

    pub unsafe fn lookup_ptr(&self, NodePtr(ptr): NodePtr<N>) -> NodeGuard<'gg, N> {
        NodeGuard {
            inside: &*ptr.as_ptr(),
            invariant: self.invariant,
        }
    }

    pub fn node(&self) -> &'gg N {
        self.inside
    }
}

pub struct NodePtr<N>(NonNull<N>);

impl<N> NodePtr<N> {
    #[allow(dead_code)]
    pub fn ptr_eq(self, other: Self) -> bool {
        std::ptr::eq(self.0.as_ptr(), other.0.as_ptr())
    }

    pub unsafe fn lookup_unchecked<'gg>(&self) -> NodeGuard<'gg, N> {
        NodeGuard {
            inside: &*self.0.as_ptr(),
            invariant: PhantomData,
        }
    }
}

impl<N> Clone for NodePtr<N> {
    fn clone(&self) -> Self {
        Self(self.0)
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
