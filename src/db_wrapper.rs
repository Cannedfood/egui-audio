use std::ops::{Deref, DerefMut};

use egui_audio::util::{from_db_deadzone, to_db_deadzone};

pub struct AsDbWrapper<'a> {
    pub original: &'a mut f32,
    pub value: f32,
    pub deadzone_db: f32,
}
impl Drop for AsDbWrapper<'_> {
    fn drop(&mut self) {
        *self.original = from_db_deadzone(self.value, self.deadzone_db);
    }
}
impl Deref for AsDbWrapper<'_> {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl DerefMut for AsDbWrapper<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

pub trait AsDbWrapperTrait<'a> {
    fn as_decibel(&'a mut self) -> AsDbWrapper<'a>;
    fn as_decibel_with_deadzone_db(&'a mut self, deadzone_db: f32) -> AsDbWrapper<'a>;
}
impl<'a> AsDbWrapperTrait<'a> for f32 {
    fn as_decibel(&'a mut self) -> AsDbWrapper<'a> {
        self.as_decibel_with_deadzone_db(-128.0)
    }

    fn as_decibel_with_deadzone_db(&'a mut self, deadzone_db: f32) -> AsDbWrapper<'a> {
        let value = to_db_deadzone(*self, deadzone_db);
        AsDbWrapper {
            original: self,
            value,
            deadzone_db,
        }
    }
}
