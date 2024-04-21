use std::num::NonZeroU32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ObjectId(NonZeroU32);

impl ObjectId {
    pub(crate) fn new(id: NonZeroU32) -> Self {
        Self(id)
    }

    pub(crate) fn get(&self) -> NonZeroU32 {
        self.0
    }

    pub(crate) fn get_zero_based_u32(&self) -> u32 {
        self.0.get() - 1
    }
}
