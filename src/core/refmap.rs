use std::panic::Location;

use crate::core::{Anchor, AnchorCore, Engine, OutputContext, Poll, UpdateContext};

// A core anchor that maps some input reference to some output reference.
///
/// Performance is critical here: `f` will always be recalled any time any downstream node
/// requests the value of this Anchor, *not* just when an input value changes.
///
/// Important: Due to constraints with Rust's lifetime system,
/// these output references can not be owned values, and must
/// live exactly as long as the input reference.
pub struct RefMap<A, F> {
    pub(super) anchors: A,
    pub(super) f: F,
    pub(super) location: &'static Location<'static>,
}

impl<A, F> RefMap<A, F> {
    pub fn new(anchors: A, f: F, location: &'static Location<'static>) -> Self {
        Self {
            anchors,
            f,
            location,
        }
    }
}

impl<F, In, Out, E> AnchorCore<E> for RefMap<(Anchor<In, E>,), F>
where
    E: Engine,
    F: for<'any> Fn(&'any In) -> &'any Out,
    In: 'static,
    Out: 'static,
{
    type Output = Out;

    fn mark_dirty(&mut self, _edge: <E::AnchorHandle as crate::core::AnchorHandle>::AnchorKey) {
        // noop
    }

    fn poll_updated(&mut self, ctx: &mut impl UpdateContext<Engine = E>) -> Poll {
        ctx.request(&self.anchors.0, true)
    }

    fn output<'slf, 'out>(
        &'slf self,
        ctx: &mut impl OutputContext<'out, Engine = E>,
    ) -> &'out Self::Output
    where
        'slf: 'out,
    {
        let val = ctx.get(&self.anchors.0);
        (self.f)(val)
    }

    fn debug_location(&self) -> Option<(&'static str, &'static Location<'static>)> {
        Some(("refmap", self.location))
    }
}
