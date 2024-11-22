use std::{iter::FromIterator, panic::Location};

use crate::expert::{
    Anchor, AnchorHandle, AnchorInner, Engine, OutputContext, Poll, UpdateContext,
};

impl<C, T, E> FromIterator<Anchor<T, E>> for Anchor<C, E>
where
    C: 'static + FromIterator<T>,
    T: 'static + Clone,
    E: Engine,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Anchor<T, E>>,
    {
        Collect::new_anchor(iter.into_iter().collect())
    }
}

impl<'a, C, T, E> FromIterator<&'a Anchor<T, E>> for Anchor<C, E>
where
    C: 'static + FromIterator<T>,
    T: 'static + Clone,
    E: Engine,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a Anchor<T, E>>,
    {
        Collect::new_anchor(iter.into_iter().cloned().collect())
    }
}

struct Collect<C, T, E: Engine> {
    anchors: Vec<Anchor<T, E>>,
    vals: Option<C>,
    location: &'static Location<'static>,
}

impl<C, T, E> Collect<C, T, E>
where
    C: 'static + FromIterator<T>,
    T: 'static + Clone,
    E: Engine,
{
    #[track_caller]
    pub fn new_anchor(anchors: Vec<Anchor<T, E>>) -> Anchor<C, E> {
        E::mount(Self {
            anchors,
            vals: None,
            location: Location::caller(),
        })
    }
}

impl<C, T, E> AnchorInner<E> for Collect<C, T, E>
where
    C: 'static + FromIterator<T>,
    T: 'static + Clone,
    E: Engine,
{
    type Output = C;

    fn dirty(&mut self, _edge: &<E::AnchorHandle as AnchorHandle>::Token) {
        self.vals = None;
    }

    fn poll_updated(&mut self, ctx: &mut impl UpdateContext<Engine = E>) -> Poll {
        if self.vals.is_none() {
            let pending_exists = self
                .anchors
                .iter()
                .any(|anchor| ctx.request(anchor, true) == Poll::Pending);
            if pending_exists {
                return Poll::Pending;
            }
            self.vals = Some(
                self.anchors
                    .iter()
                    .map(|anchor| ctx.get(anchor).clone())
                    .collect(),
            )
        }
        Poll::Updated
    }

    fn output<'slf, 'out>(
        &'slf self,
        _ctx: &mut impl OutputContext<'out, Engine = E>,
    ) -> &'out Self::Output
    where
        'slf: 'out,
    {
        self.vals.as_ref().unwrap()
    }

    fn debug_location(&self) -> Option<(&'static str, &'static Location<'static>)> {
        Some(("Collect", self.location))
    }
}

#[cfg(test)]
mod test {
    use crate::singlethread::*;

    #[test]
    fn collect() {
        let mut engine = Engine::new();

        let a = Var::new(1);
        let b = Var::new(2);
        let c = Var::new(5);

        let nums: Anchor<Vec<_>> = vec![a.watch(), b.watch(), c.watch()].into_iter().collect();
        let sum: Anchor<usize> = nums.map(|nums| nums.iter().sum());

        assert_eq!(engine.get(&sum), 8);

        a.set(2);
        assert_eq!(engine.get(&sum), 9);

        c.set(1);
        assert_eq!(engine.get(&sum), 5);
    }
}
