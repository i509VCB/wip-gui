//! Built-in widgets.
//!
//! This module provides types to describe the data of some widgets.

pub trait View {}

pub trait Ui {
    type View<'view>: View + 'view
    where
        Self: 'view;

    fn view(&mut self) -> Self::View<'_>;
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use crate::{Ui, View};

    pub struct App<'a> {
        _marker: PhantomData<&'a ()>,
        _value: (),
    }

    impl<'ui> Ui for App<'ui> {
        type View<'a> = MyView<'a>
        where
            Self: 'a;

        fn view(&mut self) -> Self::View<'_> {
            MyView {
                _value: &self._value,
            }
        }
    }

    pub struct MyView<'a> {
        _value: &'a (),
    }

    impl View for MyView<'_> {}

    pub struct App2;

    impl Ui for App2 {
        type View<'a> = View2;

        fn view(&mut self) -> Self::View<'_> {
            View2
        }
    }

    pub struct View2;
    impl View for View2 {}

    #[test]
    fn test() {}
}
