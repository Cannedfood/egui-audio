pub struct AtomicWrapper<'a, T: Copy, A: atomic_traits::Atomic<Type = T>> {
    original: &'a A,
    value: T,
}
impl<'a, T: Copy, A: atomic_traits::Atomic<Type = T>> AtomicWrapper<'a, T, A> {
    pub fn new(original: &'a A) -> Self {
        Self {
            original,
            value: original.load(std::sync::atomic::Ordering::Relaxed),
        }
    }
}
impl<'a, T: Copy, A: atomic_traits::Atomic<Type = T>> Drop for AtomicWrapper<'a, T, A> {
    fn drop(&mut self) {
        self.original
            .store(self.value, std::sync::atomic::Ordering::Relaxed);
    }
}
impl<'a, T: Copy, A: atomic_traits::Atomic<Type = T>> AsRef<T> for AtomicWrapper<'a, T, A> {
    fn as_ref(&self) -> &T {
        &self.value
    }
}

pub trait AtomicWrapperTrait<'a, T: Copy, A: atomic_traits::Atomic<Type = T>> {
    fn as_mut_ref(&'a self) -> AtomicWrapper<'a, T, A>;
}
impl<'a, T: Copy, A: atomic_traits::Atomic<Type = T>> AtomicWrapperTrait<'a, T, A> for A {
    fn as_mut_ref(&'a self) -> AtomicWrapper<'a, T, A> {
        AtomicWrapper::new(self)
    }
}
