use std::panic::Location;

use crate::expert::{Anchor, AnchorInner, Engine, OutputContext, Poll, UpdateContext};

pub struct RefMap<A, F> {
    pub(super) f: F,
    pub(super) anchors: A,
    pub(super) location: &'static Location<'static>,
}

impl<F, In: 'static, Out: 'static, E> AnchorInner<E> for RefMap<(Anchor<In, E>,), F>
where
    E: Engine,
    F: for<'any> Fn(&'any In) -> &'any Out,
{
    type Output = Out;

    fn dirty(&mut self, _edge: &<E::AnchorHandle as crate::expert::AnchorHandle>::Token) {
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
