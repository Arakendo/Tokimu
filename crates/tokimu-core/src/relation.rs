pub trait Relation: Send + Sync + 'static {}

impl<T> Relation for T where T: Send + Sync + 'static {}