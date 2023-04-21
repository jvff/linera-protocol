use {
    super::flat_type::FlatType,
    crate::layout::{FlatLayout, Layout},
    frunk::HCons,
};

pub trait MaybeFlatType: Sized {
    type Flatten<Tail: Layout>: FlatLayout;

    fn flatten<Tail>(self, tail: Tail) -> Self::Flatten<Tail>
    where
        Tail: Layout;

    fn unflatten_from<Tail>(flat_layout: Self::Flatten<Tail>) -> (Self, Tail::Flat)
    where
        Tail: Layout;

    fn unflatten<Tail>(bias: Tail::FromFlatBias, flat_layout: Self::Flatten<Tail>) -> (Self, Tail)
    where
        Tail: Layout;

    fn split_from_empty() -> Self;
    fn split_from_flat_type(flat_type: impl FlatType) -> Self;

    fn split_into<Target: MaybeFlatType>(self) -> Target;
}

impl MaybeFlatType for () {
    type Flatten<Tail: Layout> = Tail::Flat;

    fn flatten<Tail>(self, tail: Tail) -> Self::Flatten<Tail>
    where
        Tail: Layout,
    {
        tail.flatten()
    }

    fn unflatten_from<Tail>(flat_layout: Self::Flatten<Tail>) -> (Self, Tail::Flat)
    where
        Tail: Layout,
    {
        ((), flat_layout)
    }

    fn unflatten<Tail>(bias: Tail::FromFlatBias, flat_layout: Self::Flatten<Tail>) -> (Self, Tail)
    where
        Tail: Layout,
    {
        ((), Tail::from_flat(bias, flat_layout))
    }

    fn split_from_empty() {}
    fn split_from_flat_type(_flat_type: impl FlatType) {}

    fn split_into<Target: MaybeFlatType>(self) -> Target {
        Target::split_from_empty()
    }
}

impl<AllFlatTypes> MaybeFlatType for AllFlatTypes
where
    AllFlatTypes: FlatType,
{
    type Flatten<Tail: Layout> = HCons<Self, Tail::Flat>;

    fn flatten<Tail>(self, tail: Tail) -> Self::Flatten<Tail>
    where
        Tail: Layout,
    {
        HCons {
            head: self,
            tail: tail.flatten(),
        }
    }

    fn unflatten_from<Tail>(flat_layout: Self::Flatten<Tail>) -> (Self, Tail::Flat)
    where
        Tail: Layout,
    {
        (flat_layout.head, flat_layout.tail)
    }

    fn unflatten<Tail>(bias: Tail::FromFlatBias, flat_layout: Self::Flatten<Tail>) -> (Self, Tail)
    where
        Tail: Layout,
    {
        (flat_layout.head, Tail::from_flat(bias, flat_layout.tail))
    }

    fn split_from_empty() -> Self {
        unreachable!("Invalid attempt to split a flat type from an empty element");
    }

    fn split_from_flat_type(flat_type: impl FlatType) -> Self {
        flat_type.split_into()
    }

    fn split_into<Target: MaybeFlatType>(self) -> Target {
        Target::split_from_flat_type(self)
    }
}
