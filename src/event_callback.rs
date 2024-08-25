#[derive(PartialEq)]
pub enum EventCallback {
    Quit,
    Resized(u32, u32),
    Next
}
