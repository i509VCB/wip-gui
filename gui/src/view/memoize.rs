use core::marker::PhantomData;

use super::View;

pub struct Memoize<T, V, D>
where
    D: PartialEq,
    V: View<T>,
{
    data: D,
    // TODO: Copy explanation from Adapt
    f: fn(&D) -> V,
    _marker: PhantomData<T>,
}

impl<T, V, D> Memoize<T, V, D>
where
    D: PartialEq,
    V: View<T>,
{
}

impl<T, V, D> View<T> for Memoize<T, V, D>
where
    D: PartialEq,
    V: View<T>,
{
    type State = MemoizeState<T, V>;

    fn build(&self) -> Self::State {
        let view = (self.f)(&self.data);
        let state = view.build();

        MemoizeState {
            view,
            state,
            dirty: false,
        }
    }
}

pub struct MemoizeState<T, V>
where
    V: View<T>,
{
    view: V,
    state: V::State,
    dirty: bool,
}
