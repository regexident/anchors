use std::panic::Location;

use crate::core::{Anchor, AnchorCore, AnchorHandle, Engine, OutputContext, Poll, UpdateContext};

/// A core anchor that maps a number of incremental input values to some output value.
///
/// The function `f` accepts inputs as references, and must return an owned value.
/// `f` will always be recalled any time any input value changes.
pub struct Map<A, F, Out> {
    pub(super) anchors: A,
    pub(super) f: F,
    pub(super) location: &'static Location<'static>,
    pub(super) output: Option<Out>,
    pub(super) output_stale: bool,
}

impl<A, F, Out> Map<A, F, Out> {
    pub fn new(anchors: A, f: F, location: &'static Location<'static>) -> Self {
        Self {
            anchors,
            f,
            location,
            output: None,
            output_stale: true,
        }
    }
}

macro_rules! impl_tuple_map {
    ($([$output_type:ident, $num:tt])+) => {
        impl<$($output_type,)+ E, F, Out> AnchorCore<E> for
            Map<($(Anchor<$output_type, E>,)+), F, Out>
        where
            F: for<'any> FnMut($(&'any $output_type),+) -> Out,
            Out: 'static + PartialEq,
            $(
                $output_type: 'static,
            )+
            E: Engine,
        {
            type Output = Out;

            fn mark_dirty(&mut self, _edge:  <E::AnchorHandle as AnchorHandle>::AnchorKey) {
                self.output_stale = true;
            }

            fn poll_updated(
                &mut self,
                ctx: &mut impl UpdateContext<Engine=E>,
            ) -> Poll {
                if !self.output_stale && self.output.is_some() {
                    return Poll::Unchanged;
                }

                let mut found_pending = false;
                let mut found_updated = false;

                $(
                    match ctx.request(&self.anchors.$num, true) {
                        Poll::Pending => {
                            found_pending = true;
                        }
                        Poll::Updated => {
                            found_updated = true;
                        }
                        Poll::Unchanged => {
                            // do nothing
                        }
                    }
                )+

                if found_pending {
                    return Poll::Pending;
                }

                self.output_stale = false;

                if self.output.is_none() || found_updated {
                    let new_val = Some((self.f)($(&ctx.get(&self.anchors.$num)),+));
                    if new_val != self.output {
                        self.output = new_val;
                        return Poll::Updated
                    }
                }
                Poll::Unchanged
            }

            fn output<'slf, 'out>(
                &'slf self,
                _ctx: &mut impl OutputContext<'out, Engine=E>,
            ) -> &'out Self::Output
            where
                'slf: 'out,
            {
                self.output
                    .as_ref()
                    .expect("output called on Map before value was calculated")
            }

            fn debug_location(&self) -> Option<(&'static str, &'static Location<'static>)> {
                Some(("map", self.location))
            }
        }
    }
}

impl_tuple_map! {
    [O0, 0]
}

impl_tuple_map! {
    [O0, 0]
    [O1, 1]
}

impl_tuple_map! {
    [O0, 0]
    [O1, 1]
    [O2, 2]
}

impl_tuple_map! {
    [O0, 0]
    [O1, 1]
    [O2, 2]
    [O3, 3]
}

impl_tuple_map! {
    [O0, 0]
    [O1, 1]
    [O2, 2]
    [O3, 3]
    [O4, 4]
}

impl_tuple_map! {
    [O0, 0]
    [O1, 1]
    [O2, 2]
    [O3, 3]
    [O4, 4]
    [O5, 5]
}

impl_tuple_map! {
    [O0, 0]
    [O1, 1]
    [O2, 2]
    [O3, 3]
    [O4, 4]
    [O5, 5]
    [O6, 6]
}

impl_tuple_map! {
    [O0, 0]
    [O1, 1]
    [O2, 2]
    [O3, 3]
    [O4, 4]
    [O5, 5]
    [O6, 6]
    [O7, 7]
}

impl_tuple_map! {
    [O0, 0]
    [O1, 1]
    [O2, 2]
    [O3, 3]
    [O4, 4]
    [O5, 5]
    [O6, 6]
    [O7, 7]
    [O8, 8]
}
