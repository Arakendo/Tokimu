#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlatformInputEvent {
    CloseRequested,
    Resized { width: u32, height: u32 },
}
