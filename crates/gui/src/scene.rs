use core::marker::PhantomData;

use crate::view::View;

pub struct Scene<'a, N> {
    _marker: PhantomData<&'a N>,
}

impl<N> Scene<'_, N> {
    pub fn build_node<V, T>(&mut self, state: V::State) -> Node<'_, N>
    where
        V: View<T>,
        N: From<V::State>,
    {
        let node = N::from(state);
        todo!()
    }
}

pub struct Node<'a, N> {
    _marker: PhantomData<&'a N>,
}
