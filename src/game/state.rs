use super::Ship;

#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum Transition {
    Stay,
    EnterGame,
}
