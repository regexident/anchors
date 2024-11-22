use std::panic::Location;

use crate::core::{Anchor, AnchorCore, AnchorHandle, Engine, OutputContext, Poll, UpdateContext};

/// An Anchor type for immutable values.
pub struct Constant<T> {
    val: T,
    first_poll: bool,
    location: &'static Location<'static>,
}

impl<T> Constant<T>
where
    T: 'static,
{
    /// Creates a new Constant Anchor from some value.
    #[track_caller]
    #[deprecated]
    pub fn new_anchor<E>(val: T) -> Anchor<T, E>
    where
        E: Engine,
    {
        Self::new_internal(val)
    }

    pub(crate) fn new_internal<E>(val: T) -> Anchor<T, E>
    where
        E: Engine,
    {
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

impl<T, E> AnchorCore<E> for Constant<T>
where
    T: 'static,
    E: Engine,
{
    type Output = T;

    fn mark_dirty(&mut self, child: <E::AnchorHandle as AnchorHandle>::AnchorKey) {
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
