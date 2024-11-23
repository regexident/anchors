use std::panic::Location;

use crate::core::{Anchor, AnchorCore, AnchorHandle, Engine, OutputContext, Poll, UpdateContext};

/// A core anchor that outputs its input.
///
/// However, even if a value changes you may not want to recompute downstream nodes
/// unless the value changes substantially.
///
/// The function `f` accepts inputs as references, and must return true if Anchors that derive
/// values from this cutoff should recalculate, or false if derivative Anchors should not recalculate.
///
/// If this is the first calculation, `f` will be called, but return values of `false` will be ignored.
/// `f` will always be recalled any time the input value changes.
pub struct Cutoff<A, F> {
    pub(super) anchors: A,
    pub(super) f: F,
    pub(super) location: &'static Location<'static>,
}

impl<A, F> Cutoff<A, F> {
    pub fn new(anchors: A, f: F, location: &'static Location<'static>) -> Self {
        Self {
            anchors,
            f,
            location,
        }
    }
}

impl<F, In, E> AnchorCore<E> for Cutoff<(Anchor<In, E>,), F>
where
    E: Engine,
    F: for<'any> FnMut(&'any In) -> bool,
    In: 'static,
{
    type Output = In;

    fn mark_dirty(&mut self, _edge: <E::AnchorHandle as AnchorHandle>::AnchorKey) {
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
