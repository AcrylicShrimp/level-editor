use super::ObjectId;
use std::num::NonZeroU32;

#[derive(Debug)]
pub struct ObjectIdAllocator {
    next_id: NonZeroU32,
    free_ids: Vec<NonZeroU32>,
}

impl ObjectIdAllocator {
    pub(crate) fn new() -> Self {
        Self {
            next_id: NonZeroU32::MIN,
            free_ids: Vec::new(),
        }
    }

    pub(crate) fn allocate(&mut self) -> ObjectId {
        let id = match self.free_ids.pop() {
            Some(id) => id,
            None => {
                let id = self.next_id;

                match self.next_id.checked_add(1) {
                    Some(next_id) => self.next_id = next_id,
                    None => panic!("failed to allocate object id; object id overflow"),
                }

                id
            }
        };
        ObjectId::new(id)
    }

    pub(crate) fn deallocate(&mut self, id: ObjectId) {
        self.free_ids.push(id.get());
    }
}
