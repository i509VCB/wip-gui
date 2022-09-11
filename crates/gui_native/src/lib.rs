use gui::ViewNode;
use view::NativeView;

pub mod view;

pub struct Context<Data> {
    inner: gui::Context<Data, Box<dyn NativeNode<Data>>>,
}

impl<Data> Context<Data> {
    pub fn new(data: Data) -> Self {
        Self { inner: gui::Context::new(data) }
    }

    pub fn into_inner(self) -> Data {
        self.inner.into_inner()
    }
}

pub trait NativeNode<T>: ViewNode<T> {}

struct Node<T, V: NativeView<T>> {
    view: V,
    state: V::State,
}

impl<T, V> ViewNode<T> for Node<T, V>
where
    V: NativeView<T>,
{
    fn build(&mut self) {
        self.state = self.view.build();
    }
}

impl<T, V> NativeNode<T> for Node<T, V>
where
    V: NativeView<T>,
{
}

impl<T> ViewNode<T> for Box<dyn NativeNode<T>> {
    fn build(&mut self) {
        (&mut **self).build()
    }
}

impl<T> NativeNode<T> for Box<dyn NativeNode<T>> {}
