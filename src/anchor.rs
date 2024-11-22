use std::marker::PhantomData;

use crate::core::{AnchorHandle, Constant, Engine};

/// The main struct of the Anchors library. Represents a single value on the recomputation graph.
pub struct Anchor<O, E: Engine + ?Sized> {
    data: E::AnchorHandle,
    phantom: PhantomData<O>,
}

impl<O, E: Engine> Anchor<O, E> {
    #[track_caller]
    pub fn constant(value: O) -> Self
    where
        O: 'static,
    {
        Constant::new_internal(value)
    }

    /// Returns the immutable, copyable, hashable, comparable engine-specific ID for this Anchor.
    pub fn key(&self) -> <E::AnchorHandle as AnchorHandle>::AnchorKey {
        self.data.key()
    }

    pub fn new_from_core(data: E::AnchorHandle) -> Self {
        Self {
            data,
            phantom: PhantomData,
        }
    }
}

impl<O, E: Engine> Clone for Anchor<O, E> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            phantom: PhantomData,
        }
    }
}

impl<O, E: Engine> PartialEq for Anchor<O, E> {
    fn eq(&self, other: &Self) -> bool {
        self.key() == other.key()
    }
}

impl<O, E: Engine> Eq for Anchor<O, E> {}
