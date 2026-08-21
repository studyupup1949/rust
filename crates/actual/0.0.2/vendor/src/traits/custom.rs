pub trait Reset: Default {
    fn reset(&mut self) {
        *self = Self::default();
    }
}
// pub trait Drop {
//     fn drop(self) {}
// }
