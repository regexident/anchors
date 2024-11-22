use std::{cell::RefCell, rc::Rc};

use crate::core::{AnchorInner, DirtyHandle as _, Engine as _, OutputContext, Poll, UpdateContext};

use super::{Anchor, AnchorHandle, DirtyHandle, Engine};

/// A setter that can update values inside an associated `VarAnchor`.
pub struct Var<T> {
    inner: Rc<RefCell<VarShared<T>>>,
    anchor: Anchor<T>,
}

/// An Anchor type for values that are mutated by calling a setter function from outside of the Anchors recomputation graph.
struct VarAnchor<T> {
    inner: Rc<RefCell<VarShared<T>>>,
    val: Rc<T>,
}

#[derive(Clone)]
struct VarShared<T> {
    dirty_handle: Option<DirtyHandle>,
    val: Rc<T>,
    value_changed: bool,
}

impl<T> Clone for Var<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            anchor: self.anchor.clone(),
        }
    }
}

impl<T> Var<T>
where
    T: 'static,
{
    /// Creates a new Var
    pub fn new(val: T) -> Var<T> {
        let val = Rc::new(val);
        let inner = Rc::new(RefCell::new(VarShared {
            dirty_handle: None,
            val: val.clone(),
            value_changed: true,
        }));
        Var {
            inner: inner.clone(),
            anchor: Engine::mount(VarAnchor { inner, val }),
        }
    }

    /// Updates the value inside the VarAnchor, and indicates to the recomputation graph that
    /// the value has changed.
    pub fn set(&self, val: T) {
        let mut inner = self.inner.borrow_mut();
        inner.val = Rc::new(val);
        if let Some(waker) = &inner.dirty_handle {
            waker.mark_dirty();
        }
        inner.value_changed = true;
    }

    /// Retrieves the last value set
    pub fn get(&self) -> Rc<T> {
        self.inner.borrow().val.clone()
    }

    pub fn watch(&self) -> Anchor<T> {
        self.anchor.clone()
    }
}

impl<T> AnchorInner<Engine> for VarAnchor<T>
where
    T: 'static,
{
    type Output = T;

    fn mark_dirty(&mut self, _edge: &<AnchorHandle as crate::core::AnchorHandle>::Token) {
        panic!("somehow an input was dirtied on VarAnchor; it never has any inputs to dirty")
    }

    fn poll_updated(&mut self, ctx: &mut impl UpdateContext<Engine = Engine>) -> Poll {
        let mut inner = self.inner.borrow_mut();
        let first_update = inner.dirty_handle.is_none();
        if first_update {
            inner.dirty_handle = Some(ctx.dirty_handle());
        }
        let res = if inner.value_changed {
            self.val = inner.val.clone();
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
        &self.val
    }
}
