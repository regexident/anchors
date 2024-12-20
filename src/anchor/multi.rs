use std::panic::Location;

use crate::core::{AnchorCore, Cutoff, Engine, Map, MapMut, RefMap, Then};

use super::Anchor;

/// A trait automatically implemented for tuples of Anchors.
///
/// You'll likely want to `use` this trait in most of your programs, since it can create many
/// useful Anchors that derive their output incrementally from some other Anchors.
///
/// Methods here mirror the non-tuple implementations listed in [Anchor]; check that out if you're
/// curious what these methods do.
pub trait MultiAnchor<E: Engine>: Sized {
    type Target;

    fn map<F, Out>(self, f: F) -> Anchor<Out, E>
    where
        Out: 'static,
        F: 'static,
        Map<Self::Target, F, Out>: AnchorCore<E, Output = Out>;

    fn map_mut<F, Out>(self, initial: Out, f: F) -> Anchor<Out, E>
    where
        Out: 'static,
        F: 'static,
        MapMut<Self::Target, F, Out>: AnchorCore<E, Output = Out>;

    fn then<F, Out>(self, f: F) -> Anchor<Out, E>
    where
        F: 'static,
        Out: 'static,
        Then<Self::Target, Out, F, E>: AnchorCore<E, Output = Out>;

    fn cutoff<F, Out>(self, _f: F) -> Anchor<Out, E>
    where
        Out: 'static,
        F: 'static,
        Cutoff<Self::Target, F>: AnchorCore<E, Output = Out>;

    fn refmap<F, Out>(self, _f: F) -> Anchor<Out, E>
    where
        Out: 'static,
        F: 'static,
        RefMap<Self::Target, F>: AnchorCore<E, Output = Out>;
}

impl<O1, E> Anchor<O1, E>
where
    O1: 'static,
    E: Engine,
{
    /// Creates an anchor that maps a number of incremental input values to some output value.
    ///
    /// The function `f` accepts inputs as references, and must return an owned value.
    /// `f` will always be recalled any time any input value changes.
    ///
    /// This method is mirrored by [MultiAnchor::map].
    ///
    /// ```
    /// use anchors::{MultiAnchor, single_threaded::*};
    ///
    /// let mut engine = Engine::new();
    /// let a = Anchor::constant(1);
    /// let b = Anchor::constant(2);
    ///
    /// // add the two numbers together; types have been added for clarity but are optional:
    /// let res: Anchor<usize> = (&a, &b).map(|a_val: &usize, b_val: &usize| -> usize {
    ///    *a_val+*b_val
    /// });
    ///
    /// assert_eq!(3, engine.get(&res));
    /// ```
    #[track_caller]
    pub fn map<F, Out>(&self, f: F) -> Anchor<Out, E>
    where
        Out: 'static,
        F: 'static,
        Map<(Anchor<O1, E>,), F, Out>: AnchorCore<E, Output = Out>,
    {
        E::mount(Map::new((self.clone(),), f, Location::caller()))
    }

    #[track_caller]
    pub fn map_mut<F, Out>(&self, initial: Out, f: F) -> Anchor<Out, E>
    where
        Out: 'static,
        F: 'static,
        MapMut<(Anchor<O1, E>,), F, Out>: AnchorCore<E, Output = Out>,
    {
        E::mount(MapMut::new((self.clone(),), f, Location::caller(), initial))
    }

    /// Creates an anchor that maps a number of incremental input values to some output Anchor.
    ///
    /// With `then`, your computation graph can dynamically select an Anchor to recalculate based
    /// on some other incremental computation.
    ///
    /// The function `f` accepts inputs as references, and must return an owned `Anchor`.
    /// `f` will always be recalled any time any input value changes.
    ///
    /// This method is mirrored by [MultiAnchor::then].
    ///
    /// ```
    /// use anchors::{MultiAnchor, single_threaded::*};
    ///
    /// let mut engine = Engine::new();
    /// let decision = Anchor::constant(true);
    /// let num = Anchor::constant(1);
    ///
    /// // because of how we're using the `then` below, only one of these two
    /// // additions will actually be run
    /// let a = num.map(|num| *num + 1);
    /// let b = num.map(|num| *num + 2);
    ///
    /// // types have been added for clarity but are optional:
    /// let res: Anchor<usize> = decision.then(move |decision: &bool| {
    ///     if *decision {
    ///         a.clone()
    ///     } else {
    ///         b.clone()
    ///     }
    /// });
    ///
    /// assert_eq!(2, engine.get(&res));
    /// ```
    #[track_caller]
    pub fn then<F, Out>(&self, f: F) -> Anchor<Out, E>
    where
        F: 'static,
        Out: 'static,
        Then<(Anchor<O1, E>,), Out, F, E>: AnchorCore<E, Output = Out>,
    {
        E::mount(Then::new((self.clone(),), f, Location::caller()))
    }

    /// Creates an anchor that maps some input reference to some output reference.
    ///
    /// Performance is critical here: `f` will always be recalled any time any downstream node
    /// requests the value of this Anchor, *not* just when an input value changes.
    ///
    /// Important: Due to constraints with Rust's lifetime system,
    /// these output references can not be owned values, and must
    /// live exactly as long as the input reference.
    ///
    /// This method is mirrored by [MultiAnchor::refmap].
    ///
    /// ```
    /// use anchors::{MultiAnchor, single_threaded::*};
    ///
    /// struct CantClone {val: usize};
    /// let mut engine = Engine::new();
    /// let tuple = Anchor::constant((CantClone{val: 1}, CantClone{val: 2}));
    ///
    /// // lookup the first value inside the tuple; types have been added for clarity but are optional:
    /// let res: Anchor<CantClone> = tuple.refmap(|tuple: &(CantClone, CantClone)| -> &CantClone {
    ///    &tuple.0
    /// });
    ///
    /// // check if the cantclone value is correct:
    /// let is_one = res.map(|tuple: &CantClone| -> bool {
    ///    tuple.val == 1
    /// });
    ///
    /// assert_eq!(true, engine.get(&is_one));
    /// ```
    #[track_caller]
    pub fn refmap<F, Out>(&self, f: F) -> Anchor<Out, E>
    where
        Out: 'static,
        F: 'static,
        RefMap<(Anchor<O1, E>,), F>: AnchorCore<E, Output = Out>,
    {
        E::mount(RefMap::new((self.clone(),), f, Location::caller()))
    }

    /// Creates an anchor that outputs its input.
    ///
    /// However, even if a value changes you may not want to recompute downstream nodes
    /// unless the value changes substantially.
    ///
    /// The function `f` accepts inputs as references, and must return true if Anchors that derive
    /// values from this cutoff should recalculate, or false if derivative Anchors should not recalculate.
    ///
    /// If this is the first calculation, `f` will be called, but return values of `false` will be ignored.
    /// `f` will always be recalled any time the input value changes.
    ///
    /// This method is mirrored by [MultiAnchor::cutoff].
    ///
    /// ```
    /// use anchors::{MultiAnchor, single_threaded::*};
    ///
    /// let mut engine = Engine::new();
    /// let num = Variable::new(1i32);
    /// let cutoff = {
    ///     let mut old_num_opt: Option<i32> = None;
    ///     num.watch().cutoff(move |num| {
    ///         if let Some(old_num) = old_num_opt {
    ///             if (old_num - *num).abs() < 10 {
    ///                 return false;
    ///             }
    ///         }
    ///         old_num_opt = Some(*num);
    ///         true
    ///     })
    /// };
    /// let res = cutoff.map(|cutoff| *cutoff + 1);
    ///
    /// assert_eq!(2, engine.get(&res));
    ///
    /// // small changes don't cause recalculations
    /// num.set(5);
    /// assert_eq!(2, engine.get(&res));
    ///
    /// // but big changes do
    /// num.set(11);
    /// assert_eq!(12, engine.get(&res));
    /// ```
    #[track_caller]
    pub fn cutoff<F, Out>(&self, f: F) -> Anchor<Out, E>
    where
        Out: 'static,
        F: 'static,
        Cutoff<(Anchor<O1, E>,), F>: AnchorCore<E, Output = Out>,
    {
        E::mount(Cutoff::new((self.clone(),), f, Location::caller()))
    }
}

macro_rules! impl_tuple_ext {
    ($([$output_type:ident, $num:tt])+) => {
        impl <$($output_type,)+ E> Anchor<($($output_type,)+), E>
        where
            $(
                $output_type: 'static + Clone + PartialEq,
            )+
            E: Engine,
        {
            pub fn split(&self) -> ($(Anchor<$output_type, E>,)+) {
                ($(
                    self.refmap(|v| &v.$num),
                )+)
            }
        }

        impl<$($output_type,)+ E> MultiAnchor<E> for ($(&Anchor<$output_type, E>,)+)
        where
            $(
                $output_type: 'static,
            )+
            E: Engine,
        {
            type Target = ($(Anchor<$output_type, E>,)+);

            #[track_caller]
            fn map<F, Out>(self, f: F) -> Anchor<Out, E>
            where
                Out: 'static,
                F: 'static,
                Map<Self::Target, F, Out>: AnchorCore<E, Output=Out>,
            {
                E::mount(Map::new(
                    ($(self.$num.clone(),)+),
                    f,
                    Location::caller(),
                ))
            }

            #[track_caller]
            fn map_mut<F, Out>(self, initial: Out, f: F) -> Anchor<Out, E>
            where
                Out: 'static,
                F: 'static,
                MapMut<Self::Target, F, Out>: AnchorCore<E, Output=Out>,
            {
                E::mount(MapMut::new(
                    ($(self.$num.clone(),)+),
                    f,
                    Location::caller(),
                    initial,
                ))
            }

            #[track_caller]
            fn then<F, Out>(self, f: F) -> Anchor<Out, E>
            where
                F: 'static,
                Out: 'static,
                Then<Self::Target, Out, F, E>: AnchorCore<E, Output=Out>,
            {
                E::mount(Then::new(
                    ($(self.$num.clone(),)+),
                    f,
                    Location::caller(),
                ))
            }

            #[track_caller]
            fn refmap<F, Out>(self, f: F) -> Anchor<Out, E>
            where
                Out: 'static,
                F: 'static,
                RefMap<Self::Target, F>: AnchorCore<E, Output = Out>,
            {
                E::mount(RefMap::new(
                    ($(self.$num.clone(),)+),
                    f,
                    Location::caller(),
                ))
            }

            #[track_caller]
            fn cutoff<F, Out>(self, f: F) -> Anchor<Out, E>
            where
                Out: 'static,
                F: 'static,
                Cutoff<Self::Target, F>: AnchorCore<E, Output = Out>,
            {
                E::mount(Cutoff::new(
                    ($(self.$num.clone(),)+),
                    f,
                    Location::caller(),
                ))
            }
        }
    }
}

impl_tuple_ext! {
    [O0, 0]
}

impl_tuple_ext! {
    [O0, 0]
    [O1, 1]
}

impl_tuple_ext! {
    [O0, 0]
    [O1, 1]
    [O2, 2]
}

impl_tuple_ext! {
    [O0, 0]
    [O1, 1]
    [O2, 2]
    [O3, 3]
}

impl_tuple_ext! {
    [O0, 0]
    [O1, 1]
    [O2, 2]
    [O3, 3]
    [O4, 4]
}

impl_tuple_ext! {
    [O0, 0]
    [O1, 1]
    [O2, 2]
    [O3, 3]
    [O4, 4]
    [O5, 5]
}

impl_tuple_ext! {
    [O0, 0]
    [O1, 1]
    [O2, 2]
    [O3, 3]
    [O4, 4]
    [O5, 5]
    [O6, 6]
}

impl_tuple_ext! {
    [O0, 0]
    [O1, 1]
    [O2, 2]
    [O3, 3]
    [O4, 4]
    [O5, 5]
    [O6, 6]
    [O7, 7]
}

impl_tuple_ext! {
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
