use crate::flat_type::FlatType;

pub trait Sealed {}

pub trait SimpleType: Default + Sealed + Sized {
    const ALIGNMENT: u32;

    type Flat: FlatType;

    fn flatten(self) -> Self::Flat;
    fn unflatten_from(flat: Self::Flat) -> Self;
}

macro_rules! simple_type {
    ($impl_type:ident $(<$generic:ident>)? -> $flat:ty, $alignment:expr) => {
        simple_type!($impl_type -> $flat, $alignment, { |flat| flat as $impl_type });
    };

    ($impl_type:ident $(<$generic:ident>)? -> $flat:ty, $alignment:expr, $unflatten:tt) => {
        impl $(<$generic>)* Sealed for $impl_type $(<$generic>)* {}

        impl $(<$generic>)* SimpleType for $impl_type $(<$generic>)* {
            const ALIGNMENT: u32 = $alignment;

            type Flat = $flat;

            fn flatten(self) -> Self::Flat {
                self as $flat
            }

            fn unflatten_from(flat: Self::Flat) -> Self {
                let unflatten = $unflatten;
                unflatten(flat)
            }
        }
    };
}

simple_type!(bool -> i32, 1, { |flat| flat != 0 });
simple_type!(i8 -> i32, 1);
simple_type!(i16 -> i32, 2);
simple_type!(i32 -> i32, 4);
simple_type!(i64 -> i64, 8);
simple_type!(u8 -> i32, 1);
simple_type!(u16 -> i32, 2);
simple_type!(u32 -> i32, 4);
simple_type!(u64 -> i64, 8);
simple_type!(f32 -> f32, 4);
simple_type!(f64 -> f64, 8);
simple_type!(char -> i32, 4,
    { |flat| char::from_u32(flat as u32).expect("Attempt to unflatten an invalid `char`") }
);
