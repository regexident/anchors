use std::{panic::Location, rc::Rc};

use crate::core::{AnchorCore, Engine as _, OutputContext, Poll, UpdateContext};

use super::{Anchor, AnchorHandle, Engine};

/// A constant that exposes an anchor for its value.
pub struct Constant<T> {
    value: Rc<T>,
    anchor: Anchor<T>,
}

impl<T> Clone for Constant<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            anchor: self.anchor.clone(),
        }
    }
}

impl<T> Constant<T>
where
    T: 'static,
{
    /// Creates a new Const
    #[track_caller]
    pub fn new(value: T) -> Constant<T> {
        let value = Rc::new(value);
        Constant {
            value: Rc::clone(&value),
            anchor: Engine::mount(ConstAnchor::new(value, Location::caller())),
        }
    }

    /// Retrieves the value
    pub fn get(&self) -> Rc<T> {
        self.value.clone()
    }

    pub fn watch(&self) -> Anchor<T> {
        self.anchor.clone()
    }

    pub fn into_anchor(self) -> Anchor<T> {
        self.anchor
    }
}

/// An Anchor type for values that are mutated by calling a setter function from outside of the Anchors recomputation graph.
pub(super) struct ConstAnchor<T> {
    value: Rc<T>,
    location: &'static Location<'static>,
    first_poll: bool,
}

impl<T> ConstAnchor<T> {
    pub(super) fn new(value: Rc<T>, location: &'static Location<'static>) -> Self {
        Self {
            value,
            location,
            first_poll: true,
        }
    }
}

impl<T> AnchorCore<Engine> for ConstAnchor<T>
where
    T: 'static,
{
    type Output = T;

    fn mark_dirty(&mut self, _edge: <AnchorHandle as crate::core::AnchorHandle>::AnchorKey) {
        panic!("attempt to mark a constant's non-existent inputs as as dirty")
    }

    fn poll_updated(&mut self, _ctx: &mut impl UpdateContext<Engine = Engine>) -> Poll {
        let poll = if self.first_poll {
            Poll::Updated
        } else {
            Poll::Unchanged
        };
        self.first_poll = false;
        poll
    }

    fn output<'slf, 'out>(
        &'slf self,
        _ctx: &mut impl OutputContext<'out, Engine = Engine>,
    ) -> &'out Self::Output
    where
        'slf: 'out,
    {
        &self.value
    }

    fn debug_location(&self) -> Option<(&'static str, &'static Location<'static>)> {
        Some(("Constant", self.location))
    }
}
