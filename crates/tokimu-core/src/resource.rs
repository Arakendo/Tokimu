pub trait Resource: Send + Sync + 'static {}

impl<T> Resource for T where T: Send + Sync + 'static {}
