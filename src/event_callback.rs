#[derive(PartialEq)]
pub enum EventCallback {
    QUIT,
    RESIZE(u32, u32),
    NEXT
}
