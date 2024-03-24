use std::ops::{Deref, DerefMut};

pub struct AsScaledWrapper<'a> {
    pub original: &'a mut f32,
    pub value:    f32,
    pub factor:   f32,
}
impl Drop for AsScaledWrapper<'_> {
    fn drop(&mut self) { *self.original = self.value / self.factor; }
}
impl Deref for AsScaledWrapper<'_> {
    type Target = f32;

    fn deref(&self) -> &Self::Target { &self.value }
}
impl DerefMut for AsScaledWrapper<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.value }
}

pub trait AsScaledWrapperTrait<'a> {
    fn as_scaled_by(&'a mut self, factor: Self) -> AsScaledWrapper<'a>;
}
impl<'a> AsScaledWrapperTrait<'a> for f32 {
    fn as_scaled_by(&'a mut self, factor: Self) -> AsScaledWrapper<'a> {
        let value = *self * factor;
        AsScaledWrapper {
            original: self,
            factor,
            value,
        }
    }
}
