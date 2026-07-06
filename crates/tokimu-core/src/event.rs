pub trait Event: Send + Sync + 'static {}

impl<T> Event for T where T: Send + Sync + 'static {}
