#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum Transition {
    Stay,
    EnterGame,
}

// TODO: probably
// pub(crate) enum LifeCycle {
//     Born,
//     Living,
//     Dead,
// }
// pub(crate) struct ObjLife<T> {
//     inner: T,
//     life_cycle: LifeCycle,
// }
