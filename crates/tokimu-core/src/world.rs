use crate::EntityId;

#[derive(Debug, Default)]
pub struct World {
    next_entity_id: u64,
}

impl World {
    pub fn spawn(&mut self) -> EntityId {
        let id = EntityId(self.next_entity_id);
        self.next_entity_id += 1;
        id
    }
}
