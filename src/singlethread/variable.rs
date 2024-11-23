use std::{cell::RefCell, panic::Location, rc::Rc};

use crate::core::{AnchorCore, DirtyHandle as _, Engine as _, OutputContext, Poll, UpdateContext};

use super::{Anchor, AnchorHandle, DirtyHandle, Engine};

/// A variable that exposes an anchor for its value.
pub struct Variable<T> {
    inner: Rc<RefCell<VarShared<T>>>,
    anchor: Anchor<T>,
}

impl<T> Clone for Variable<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            anchor: self.anchor.clone(),
        }
    }
}

impl<T> Variable<T>
where
    T: 'static,
{
    /// Creates a new variable
    #[track_caller]
    pub fn new(value: T) -> Variable<T> {
        let value = Rc::new(value);
        let inner = Rc::new(RefCell::new(VarShared {
            dirty_handle: None,
            value: value.clone(),
            value_changed: true,
        }));
        Variable {
            inner: inner.clone(),
            anchor: Engine::mount(VarAnchor {
                inner,
                value,
                location: Location::caller(),
            }),
        }
    }

    /// Updates the value inside the VarAnchor, and indicates to the recomputation graph that
    /// the value has changed.
    pub fn set(&self, value: T) {
        let mut inner = self.inner.borrow_mut();
        inner.value = Rc::new(value);
        if let Some(waker) = &inner.dirty_handle {
            waker.mark_dirty();
        }
        inner.value_changed = true;
    }

    /// Retrieves the last value set
    pub fn get(&self) -> Rc<T> {
        self.inner.borrow().value.clone()
    }

    pub fn watch(&self) -> Anchor<T> {
        self.anchor.clone()
    }
}

#[derive(Clone)]
struct VarShared<T> {
    dirty_handle: Option<DirtyHandle>,
    value: Rc<T>,
    value_changed: bool,
}

/// An Anchor type for values that are mutated by calling a setter function from outside of the Anchors recomputation graph.
struct VarAnchor<T> {
    inner: Rc<RefCell<VarShared<T>>>,
    value: Rc<T>,
    location: &'static Location<'static>,
}

impl<T> AnchorCore<Engine> for VarAnchor<T>
where
    T: 'static,
{
    type Output = T;

    fn mark_dirty(&mut self, _edge: <AnchorHandle as crate::core::AnchorHandle>::AnchorKey) {
        panic!("attempt to mark a variable's non-existent inputs as as dirty")
    }

    fn poll_updated(&mut self, ctx: &mut impl UpdateContext<Engine = Engine>) -> Poll {
        let mut inner = self.inner.borrow_mut();
        let first_update = inner.dirty_handle.is_none();
        if first_update {
            inner.dirty_handle = Some(ctx.dirty_handle());
        }
        let res = if inner.value_changed {
            self.value = inner.value.clone();
            Poll::Updated
        } else {
            Poll::Unchanged
        };
        inner.value_changed = false;
        res
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
        Some(("Variable", self.location))
    }
}
