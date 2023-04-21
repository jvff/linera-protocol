use super::simple_type::SimpleType;

pub trait FlatType: SimpleType<Flat = Self> {
    fn split_from_i32(joined_i32: i32) -> Self;
    fn split_from_i64(joined_i64: i64) -> Self;
    fn split_from_f32(joined_f32: f32) -> Self;
    fn split_from_f64(joined_f64: f64) -> Self;

    fn split_into<Target: FlatType>(self) -> Target;
}

impl FlatType for i32 {
    fn split_from_i32(joined_i32: i32) -> Self {
        joined_i32
    }

    fn split_from_i64(joined_i64: i64) -> Self {
        joined_i64
            .try_into()
            .expect("Invalid `i32` stored in `i64`")
    }

    fn split_from_f32(_joined_f32: f32) -> Self {
        unreachable!("`i32` is never joined into `f32`");
    }

    fn split_from_f64(_joined_f64: f64) -> Self {
        unreachable!("`i32` is never joined into `f64`");
    }

    fn split_into<Target: FlatType>(self) -> Target {
        Target::split_from_i32(self)
    }
}

impl FlatType for i64 {
    fn split_from_i32(_joined_i32: i32) -> Self {
        unreachable!("`i64` is never joined into `i32`");
    }

    fn split_from_i64(joined_i64: i64) -> Self {
        joined_i64
    }

    fn split_from_f32(_joined_f32: f32) -> Self {
        unreachable!("`i64` is never joined into `f32`");
    }

    fn split_from_f64(_joined_f64: f64) -> Self {
        unreachable!("`i64` is never joined into `f64`");
    }

    fn split_into<Target: FlatType>(self) -> Target {
        Target::split_from_i64(self)
    }
}

impl FlatType for f32 {
    fn split_from_i32(joined_i32: i32) -> Self {
        joined_i32 as f32
    }

    fn split_from_i64(joined_i64: i64) -> Self {
        (joined_i64 as i32) as f32
    }

    fn split_from_f32(joined_f32: f32) -> Self {
        joined_f32
    }

    fn split_from_f64(_joined_f64: f64) -> Self {
        unreachable!("`f32` is never joined into `f64`");
    }

    fn split_into<Target: FlatType>(self) -> Target {
        Target::split_from_f32(self)
    }
}

impl FlatType for f64 {
    fn split_from_i32(_joined_i32: i32) -> Self {
        unreachable!("`f64` is never joined into `i32`");
    }

    fn split_from_i64(joined_i64: i64) -> Self {
        joined_i64 as f64
    }

    fn split_from_f32(_joined_f32: f32) -> Self {
        unreachable!("`f64` is never joined into `f32`");
    }

    fn split_from_f64(joined_f64: f64) -> Self {
        joined_f64
    }

    fn split_into<Target: FlatType>(self) -> Target {
        Target::split_from_f64(self)
    }
}
