use std::panic::Location;

use crate::expert::{
    Anchor, AnchorHandle, AnchorInner, Engine, OutputContext, Poll, UpdateContext,
};

/// An Anchor type for immutable values.
pub struct Constant<T> {
    val: T,
    first_poll: bool,
    location: &'static Location<'static>,
}

impl<T: 'static> Constant<T> {
    /// Creates a new Constant Anchor from some value.
    #[track_caller]
    #[deprecated]
    pub fn new_anchor<E: Engine>(val: T) -> Anchor<T, E> {
        Self::new_internal(val)
    }

    pub(crate) fn new_internal<E: Engine>(val: T) -> Anchor<T, E> {
        E::mount(Self {
            val,
            first_poll: true,
            location: Location::caller(),
        })
    }

    #[cfg(test)]
    pub fn new_raw_testing(val: T) -> Constant<T> {
        Self {
            val,
            first_poll: true,
            location: Location::caller(),
        }
    }
}

impl<T: 'static, E: Engine> AnchorInner<E> for Constant<T> {
    type Output = T;

    fn dirty(&mut self, child: &<E::AnchorHandle as AnchorHandle>::Token) {
        panic!(
            "Constant never has any inputs; dirty should not have been called. alleged child: {:?}",
            child
        )
    }

    fn poll_updated(&mut self, _ctx: &mut impl UpdateContext<Engine = E>) -> Poll {
        let res = if self.first_poll {
            Poll::Updated
        } else {
            Poll::Unchanged
        };
        self.first_poll = false;
        res
    }

    fn output<'slf, 'out>(
        &'slf self,
        _ctx: &mut impl OutputContext<'out, Engine = E>,
    ) -> &'out Self::Output
    where
        'slf: 'out,
    {
        &self.val
    }

    fn debug_location(&self) -> Option<(&'static str, &'static Location<'static>)> {
        Some(("constant", self.location))
    }
}
