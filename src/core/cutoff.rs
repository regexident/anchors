use std::panic::Location;

use crate::core::{Anchor, AnchorHandle, AnchorCore, Engine, OutputContext, Poll, UpdateContext};

pub struct Cutoff<A, F> {
    pub(super) f: F,
    pub(super) anchors: A,
    pub(super) location: &'static Location<'static>,
}

impl<F, In, E> AnchorCore<E> for Cutoff<(Anchor<In, E>,), F>
where
    E: Engine,
    F: for<'any> FnMut(&'any In) -> bool,
    In: 'static,
{
    type Output = In;

    fn mark_dirty(&mut self, _edge: &<E::AnchorHandle as AnchorHandle>::Token) {
        // noop
    }

    fn poll_updated(&mut self, ctx: &mut impl UpdateContext<Engine = E>) -> Poll {
        let upstream_poll = ctx.request(&self.anchors.0, true);
        if upstream_poll != Poll::Updated {
            return upstream_poll;
        }

        let val = ctx.get(&self.anchors.0);
        if (self.f)(val) {
            Poll::Updated
        } else {
            Poll::Unchanged
        }
    }

    fn output<'slf, 'out>(
        &'slf self,
        ctx: &mut impl OutputContext<'out, Engine = E>,
    ) -> &'out Self::Output
    where
        'slf: 'out,
    {
        ctx.get(&self.anchors.0)
    }

    fn debug_location(&self) -> Option<(&'static str, &'static Location<'static>)> {
        Some(("cutoff", self.location))
    }
}
