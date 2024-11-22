//! Common code used between different engines.
//!
//! Unless you're implementing your own generic `AnchorInner`s or your own execution engine,
//! you should never need to import things from here. `singlethread` should re-export anything
//! you need to use `anchors`!

use std::{fmt::Debug, hash::Hash, panic::Location};

mod var;

use crate::Anchor;

mod constant;
mod cutoff;
mod map;
mod map_mut;
mod multi;
mod refmap;
mod then;

pub use self::{constant::*, cutoff::*, map::*, map_mut::*, multi::*, refmap::*, then::*};

/// Indicates whether a value is ready for reading, and if it is, whether it's changed
/// since the last read.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Poll {
    /// Indicates the polled value is ready for reading. Either this is the first read,
    /// or the value has changed since the last read.
    Updated,

    /// Indicates the polled value is ready for reading. This is not the first read, and
    /// the value is unchanged since the previous read.
    Unchanged,

    /// Indicates the polled value is not ready for reading, but has been queued for recalculation.
    /// The output value will eventually switch to Updated or Unchanged.
    Pending,
}

/// A reference to a particular `AnchorInner`. Each engine implements its own.
pub trait AnchorHandle: Sized + Clone {
    type Token: Sized + Clone + Copy + PartialEq + Eq + Hash + Debug;

    /// Returns a Copyable, comparable, hashable ID corresponding to this AnchorHandle.
    /// Some engines may garbage collect an AnchorInner when no more AnchorHandles pointing
    /// to it exist, which means it's possible to have a Token pointing to a since-deleted
    /// Anchor.
    fn token(&self) -> Self::Token;
}

/// The core engine trait implemented by each recalculation engine. Allows mounting an `AnchorInner`
/// into an actual `Anchor`, although this mounting should usually be done by each `AnchorInner`
/// implementation directly.
pub trait Engine: 'static {
    type AnchorHandle: AnchorHandle;
    type DirtyHandle: DirtyHandle;

    fn mount<I>(inner: I) -> Anchor<I::Output, Self>
    where
        I: 'static + AnchorInner<Self>;
}

/// Allows a node with non-Anchors inputs to manually mark itself as dirty. Each engine implements its own.
pub trait DirtyHandle {
    /// Indicates that the Anchor associated with this `DirtyHandle` may have a changed its output, and should
    /// be re-polled.
    fn mark_dirty(&self);
}

/// The context passed to an `AnchorInner` when its `output` method is called.
pub trait OutputContext<'eng> {
    type Engine: Engine + ?Sized;

    /// If another Anchor during polling indicated its value was ready, the previously
    /// calculated value can be accessed with this method. Its implementation is virtually
    /// identical to `UpdateContext`'s `get`. This is mostly used by AnchorInner implementations
    /// that want to return a reference to some other Anchor's output without cloning.
    fn get<'out, O>(&self, anchor: &Anchor<O, Self::Engine>) -> &'out O
    where
        'eng: 'out,
        O: 'static;
}

/// The context passed to an `AnchorInner` when its `poll_updated` method is called.
pub trait UpdateContext {
    type Engine: Engine + ?Sized;

    /// If `request` indicates another Anchor's value is ready, the previously
    /// calculated value can be accessed with this method.
    fn get<'out, 'slf, O>(&'slf self, anchor: &Anchor<O, Self::Engine>) -> &'out O
    where
        'slf: 'out,
        O: 'static;

    /// If `anchor`'s output is ready, indicates whether the output has changed since this `AnchorInner`
    /// last called `request` on it. If `anchor`'s output is not ready, it is queued for recalculation and
    /// this returns Poll::Pending.
    ///
    /// `necessary` is a bit that indicates if we are necessary, `anchor` should be marked as necessary
    /// as well. If you don't know what this bit should be set to, you probably want a value of `true`.
    fn request<O>(&mut self, anchor: &Anchor<O, Self::Engine>, necessary: bool) -> Poll
    where
        O: 'static;

    /// If `anchor` was previously passed to `request` and you no longer care about its output, you can
    /// pass it to `unrequest` so the engine will stop calling your `dirty` method when `anchor` changes.
    /// If `self` is necessary, this is also critical for ensuring `anchor` is no longer marked as necessary.
    fn unrequest<O>(&mut self, anchor: &Anchor<O, Self::Engine>)
    where
        O: 'static;

    /// Returns a new dirty handle, used for marking that `self`'s output may have changed through some
    /// non incremental means. For instance, perhaps this `AnchorInner`s value represents the current time, or
    /// it's a `Var` that has a setter function.
    fn dirty_handle(&mut self) -> <Self::Engine as Engine>::DirtyHandle;
}

/// The engine-agnostic implementation of each type of Anchor. You likely don't need to implement your own
/// `AnchorInner`; instead use one of the built-in implementations.
pub trait AnchorInner<E>
where
    E: Engine + ?Sized,
{
    type Output;

    /// Called by the engine to indicate some input may have changed.
    /// If this `AnchorInner` still cares about `child`'s value, it should re-request
    /// it next time `poll_updated` is called.
    fn mark_dirty(&mut self, child: &<E::AnchorHandle as AnchorHandle>::Token);

    /// Called by the engine when it wants to know if this value has changed or
    /// not. If some requested value from `ctx` is `Pending`, this method should
    /// return `Poll::Pending`; otherwise it must finish recalculation and report
    /// either `Poll::Updated` or `Poll::Unchanged`.
    fn poll_updated(&mut self, ctx: &mut impl UpdateContext<Engine = E>) -> Poll;

    /// Called by the engine to get the current output value of this `AnchorInner`. This
    /// is *only* called after this `AnchorInner` reported in the return value from
    /// `poll_updated` the value was ready. If `dirty` is called, this function will not
    /// be called until `poll_updated` returns a non-Pending value.
    fn output<'slf, 'out>(
        &'slf self,
        ctx: &mut impl OutputContext<'out, Engine = E>,
    ) -> &'out Self::Output
    where
        'slf: 'out;

    /// An optional function to report the track_caller-derived call-site where
    /// this Anchor was created. Useful for debugging purposes.
    fn debug_location(&self) -> Option<(&'static str, &'static Location<'static>)> {
        None
    }
}
