#[derive(Clone, Copy, PartialEq)]
pub enum RenderMode {
    Reactive,
    Continious,
}

impl Default for RenderMode {
    fn default() -> Self {
        if cfg!(debug_assertions) {
            Self::Continious
        } else {
            Self::Reactive
        }
    }
}
