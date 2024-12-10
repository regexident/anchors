use std::{cell::RefMut, marker::PhantomData};

use super::{NodeGuard, NodePtr};

pub(super) struct RefCellVecIterator<'a> {
    inside: RefMut<'a, Vec<NodePtr>>,
    next_i: usize,
    first: Option<NodePtr>,
    empty_on_drop: bool,
    // hack to make RefCellVecIterator invariant
    f: PhantomData<&'a mut &'a ()>,
}

impl<'a> RefCellVecIterator<'a> {
    pub(super) fn new(
        inside: RefMut<'a, Vec<NodePtr>>,
        next_i: usize,
        first: Option<NodePtr>,
        empty_on_drop: bool,
    ) -> Self {
        Self {
            inside,
            next_i,
            first,
            empty_on_drop,
            f: PhantomData,
        }
    }
}

impl<'a> Iterator for RefCellVecIterator<'a> {
    type Item = NodeGuard<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(first) = self.first.take() {
            return Some(NodeGuard(unsafe { first.lookup_unchecked() }));
        }
        let next = self.inside.get(self.next_i)?;
        self.next_i += 1;
        Some(NodeGuard(unsafe { next.lookup_unchecked() }))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let mut remaining = self.inside.len() - self.next_i;

        if self.first.is_some() {
            remaining += 1;
        }

        (remaining, Some(remaining))
    }
}

impl Drop for RefCellVecIterator<'_> {
    fn drop(&mut self) {
        if self.empty_on_drop {
            self.inside.clear()
        }
    }
}
