// #![feature(specialization)]

// trait Float{}
// struct Vector<T,U> {

// }

// trait VectorExt<T>
// where
//     T: Float,
// {
//     fn length(&self) -> T;
// }

// impl<T, Type> VectorExt<T> for Vector<T, Type>
// where
//     T: Float,
// {
//     default fn length(&self) -> T {
//         println!("NON SPECIAL");
//         T::one()
//     }
// }

// impl<T> VectorExt<T> for Vector<T, Unit>
// where
//     T: Float,
// {
//     fn length(&self) -> T {
//         println!("SPECIAL");
//         T::one()
//     }
// }

// // This `impl` is not strictly necessary,
// // but it will let users of your type
// // use the `length` method
// // without having to `use` the `VectorExt` trait.
// impl<T, Type> Vector<T, Type>
// where
//     T: Float,
// {
//     fn length(&self) -> T
//     where
//         Self: VectorExt<T>,
//     {
//         VectorExt::<T>::length(self)
//         // can also be written as: <Self as VectorExt<T>>::length(self)
//     }
// }
