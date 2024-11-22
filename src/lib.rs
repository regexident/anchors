mod collections;
pub mod core;
pub mod single_threaded;

mod anchor;
mod arena;

pub mod prelude {
    pub use crate::core::{
        AnchorCore, AnchorHandle, DirtyHandle, Engine, OutputContext, UpdateContext,
    };
    pub use crate::{Anchor, MultiAnchor};
}

pub use self::anchor::*;
