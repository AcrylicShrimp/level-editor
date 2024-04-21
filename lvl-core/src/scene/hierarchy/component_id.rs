use std::num::NonZeroU32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ComponentId(NonZeroU32);

impl ComponentId {
    pub(crate) fn new(id: NonZeroU32) -> Self {
        Self(id)
    }

    pub(crate) fn get(&self) -> NonZeroU32 {
        self.0
    }
}
