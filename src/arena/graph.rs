use std::marker::PhantomData;

use typed_arena::Arena;

use super::graph_guard::GraphGuard;

pub struct Graph<N> {
    pub(super) arena: Arena<N>,
}

impl<N> Graph<N> {
    pub fn new() -> Self {
        Graph {
            arena: Arena::new(),
        }
    }

    pub fn with<R>(&self, f: impl for<'any> FnOnce(GraphGuard<'any, N>) -> R) -> R {
        f(GraphGuard {
            graph: self,
            invariant: PhantomData,
        })
    }

    pub unsafe fn with_unchecked<'gg, 'slf>(&'slf self) -> GraphGuard<'gg, N>
    where
        'slf: 'gg,
    {
        GraphGuard {
            graph: self,
            invariant: PhantomData,
        }
    }
}

impl<N> Default for Graph<N> {
    fn default() -> Self {
        Self::new()
    }
}
