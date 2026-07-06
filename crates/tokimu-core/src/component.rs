pub trait Component: Send + Sync + 'static {}

impl<T> Component for T where T: Send + Sync + 'static {}
