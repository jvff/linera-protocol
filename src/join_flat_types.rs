use {super::maybe_flat_type::MaybeFlatType, either::Either, int_conv::ZeroExtend};

pub trait JoinFlatTypes {
    type Flat: MaybeFlatType;

    fn join(self) -> Self::Flat;
}

macro_rules! join_flat_types {
    (
        $( ($left:ty, $right:ty) -> $joined:ty => ($join_left:tt, $join_right:tt $(,)?) ),* $(,)?
    ) => {
        $(
            impl JoinFlatTypes for Either<$left, $right> {
                type Flat = $joined;

                fn join(self) -> Self::Flat {
                    match self {
                        Either::Left(left) => {
                            let join_left = $join_left;
                            join_left(left)
                        }
                        Either::Right(right) => {
                            let join_right = $join_right;
                            join_right(right)
                        }
                    }
                }
            }
        )*
    }
}

join_flat_types!(
    (i32, i32) -> i32 => (
        { |value| value },
        { |value| value },
    ),
    (i32, i64) -> i64 => (
        { |value: i32| value.zero_extend() },
        { |value| value },
    ),
    (i32, f32) -> i32 => (
        { |value| value },
        { |value| value as i32 },
    ),
    (i32, f64) -> i64 => (
        { |value: i32| value.zero_extend() },
        { |value| value as i64 },
    ),

    (i64, i32) -> i64 => (
        { |value| value },
        { |value: i32| value.zero_extend() },
    ),
    (i64, i64) -> i64 => (
        { |value| value },
        { |value| value },
    ),
    (i64, f32) -> i64 => (
        { |value| value },
        { |value| (value as i32).zero_extend() },
    ),
    (i64, f64) -> i64 => (
        { |value| value },
        { |value| value as i64 },
    ),

    (f32, i32) -> i32 => (
        { |value| value as i32 },
        { |value| value },
    ),
    (f32, i64) -> i64 => (
        { |value| (value as i32).zero_extend() },
        { |value| value },
    ),
    (f32, f32) -> f32 => (
        { |value| value },
        { |value| value },
    ),
    (f32, f64) -> i64 => (
        { |value| (value as i32).zero_extend() },
        { |value| value as i64 },
    ),

    (f64, i32) -> i64 => (
        { |value| value as i64 },
        { |value: i32| value.zero_extend() },
    ),
    (f64, i64) -> i64 => (
        { |value| value as i64 },
        { |value| value },
    ),
    (f64, f32) -> i64 => (
        { |value| value as i64 },
        { |value| (value as i32).zero_extend() },
    ),
    (f64, f64) -> f64 => (
        { |value| value },
        { |value| value },
    ),
);
