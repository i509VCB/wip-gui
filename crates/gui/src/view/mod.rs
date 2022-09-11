use core::{marker::PhantomData, num::NonZeroU64};

// pub mod adapt;
// pub mod memoize;

pub struct SlotIdentity<T> {
    id: NonZeroU64,
    _marker: PhantomData<T>,
}

#[must_use]
pub struct Slot<'a, V, T>
where
    V: View<T>,
{
    state: &'a mut Option<V::State>,
}

pub trait View<T>: Sized {
    type State: Sized;

    fn build(&self, slot: Slot<'_, Self, T>) -> SlotIdentity<Self::State>;
}
