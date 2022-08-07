use super::View;

pub struct Adapt<T, U, V>
where
    V: View<U>,
{
    view: V,
    // This could be a generic type, `F: Fn(&mut T) -> &mut U` but that introduces an existential `impl Fn`
    // type which cannot be written currently.
    //
    // Box<dyn Fn(&mut T) -> &mut R> could also work. This has the advantage of removing the generic type, but
    // introduces indirection and dynamic dispatch. If the closure captures then the Box will require an
    // allocation.
    //
    // Using a function pointer means no allocation or existential generic type. This however means closures
    // cannot capture anything. Given `&mut T` is available for the mapping, it covers that case.
    //
    // Ideally the true solution to this probably be existential types to allow naming the type, capture in
    // closures captures and avoid indirection.
    f: for<'r> fn(&'r mut T) -> &'r mut U,
}

impl<T, U, V> Adapt<T, U, V>
where
    V: View<U>,
{
    pub fn new(view: V, f: for<'r> fn(&'r mut T) -> &'r mut U) -> Self {
        Self { view, f }
    }
}

impl<T, U, V> core::fmt::Debug for Adapt<T, U, V>
where
    V: View<U> + core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Adapt")
            .field(&self.view)
            .field(&format_args!(
                "{} -> {}",
                core::any::type_name::<T>(),
                core::any::type_name::<U>()
            ))
            .finish()
    }
}

impl<T, U, V> View<T> for Adapt<T, U, V>
where
    V: View<U>,
{
    type State = V::State;

    fn build(&self) -> Self::State {
        self.view.build()
    }
}
