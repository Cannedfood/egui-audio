use std::ops::{Deref, DerefMut};

pub struct AtomicWrapper<'a, T, A>
where
    T: Copy + PartialEq,
    A: atomic_traits::Atomic<Type = T>,
{
    original: &'a A,
    value:    T,
}
impl<'a, T, A> AtomicWrapper<'a, T, A>
where
    T: Copy + PartialEq,
    A: atomic_traits::Atomic<Type = T>,
{
    pub fn new(original: &'a A) -> Self {
        let value = original.load(std::sync::atomic::Ordering::Relaxed);
        Self { original, value }
    }
}
impl<'a, T, A> Drop for AtomicWrapper<'a, T, A>
where
    T: Copy + PartialEq,
    A: atomic_traits::Atomic<Type = T>,
{
    fn drop(&mut self) {
        self.original
            .store(self.value, std::sync::atomic::Ordering::Relaxed);
    }
}
impl<'a, T, A> Deref for AtomicWrapper<'a, T, A>
where
    T: Copy + PartialEq,
    A: atomic_traits::Atomic<Type = T>,
{
    type Target = T;

    fn deref(&self) -> &Self::Target { &self.value }
}
impl<'a, T, A> DerefMut for AtomicWrapper<'a, T, A>
where
    T: Copy + PartialEq,
    A: atomic_traits::Atomic<Type = T>,
{
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.value }
}

pub trait AtomicWrapperTrait<'a, T, A>
where
    T: Copy + PartialEq,
    A: atomic_traits::Atomic<Type = T>,
{
    fn read_modify_write(&'a self) -> AtomicWrapper<'a, T, A>;
}
impl<'a, T, A> AtomicWrapperTrait<'a, T, A> for A
where
    T: Copy + PartialEq,
    A: atomic_traits::Atomic<Type = T>,
{
    fn read_modify_write(&'a self) -> AtomicWrapper<'a, T, A> { AtomicWrapper::new(self) }
}
