use std::{any::Any, panic::Location};

use crate::core::{AnchorCore, Poll};

use super::{Anchor, AnchorKey, Constant, Engine, EngineContext, EngineContextMut};

impl<T> Anchor<T> {
    /// A constant value's anchor without a corresponding `Constant`.
    #[track_caller]
    pub fn constant(value: T) -> Self
    where
        T: 'static,
    {
        Constant::new(value).into_anchor()
    }
}

pub(super) trait GenericAnchor {
    fn mark_dirty(&mut self, child_key: AnchorKey);

    fn poll_updated(&mut self, ctx: &mut EngineContextMut<'_, '_>) -> Poll;

    fn output<'slf, 'out>(&'slf self, ctx: &mut EngineContext<'out>) -> &'out dyn Any
    where
        'slf: 'out;

    fn debug_info(&self) -> AnchorDebugInfo;
}

impl<I> GenericAnchor for I
where
    I: 'static + AnchorCore<Engine>,
{
    fn mark_dirty(&mut self, child_key: AnchorKey) {
        AnchorCore::mark_dirty(self, child_key)
    }

    fn poll_updated(&mut self, ctx: &mut EngineContextMut<'_, '_>) -> Poll {
        AnchorCore::poll_updated(self, ctx)
    }

    fn output<'slf, 'out>(&'slf self, ctx: &mut EngineContext<'out>) -> &'out dyn Any
    where
        'slf: 'out,
    {
        AnchorCore::output(self, ctx)
    }

    fn debug_info(&self) -> AnchorDebugInfo {
        AnchorDebugInfo {
            location: self.debug_location(),
            type_info: std::any::type_name::<I>(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub(super) struct AnchorDebugInfo {
    pub(super) location: Option<(&'static str, &'static Location<'static>)>,
    pub(super) type_info: &'static str,
}

impl std::fmt::Display for AnchorDebugInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.location {
            Some((name, location)) => write!(f, "{location} ({name})"),
            None => write!(f, "{}", self.type_info),
        }
    }
}
