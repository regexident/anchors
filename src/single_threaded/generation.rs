use std::num::NonZeroU64;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub(super) struct Generation(NonZeroU64);

impl Generation {
    pub(super) fn new() -> Generation {
        Generation(NonZeroU64::new(1).unwrap())
    }

    pub(super) fn increment(&mut self) {
        let gen: u64 = u64::from(self.0).wrapping_add(1);
        self.0 = NonZeroU64::new(gen).unwrap();
    }
}
