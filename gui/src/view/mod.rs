pub mod adapt;
pub mod memoize;

pub trait View<T>: Sized {
    type State: Sized;

    fn build(&self) -> Self::State;
}
