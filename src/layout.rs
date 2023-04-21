use {
    super::{flat_type::FlatType, layout_element::LayoutElement, maybe_flat_type::MaybeFlatType},
    crate::util::Merge,
    frunk::{hlist::HList, HCons, HNil},
};

pub trait Sealed {}

pub trait Layout: Sealed + HList {
    const ALIGNMENT: u32;
    const COUNT: u32;

    type Flat: FlatLayout;
    type FromFlatBias: Clone + Default + HList;
    type Merge<Other: Layout>: Layout
    where
        Self: Merge<Other>,
        <Self as Merge<Other>>::Output: Layout;

    fn from_flat(bias: Self::FromFlatBias, flat_layout: Self::Flat) -> Self;

    // fn unmerge_right_from_flat<Other>(
    // flat_layout: <<Other as Merge<Self>>::Output as Layout>::Flat,
    // ) -> Self
    // where
    // Other: Merge<Self>,
    // Other::Output: Layout;

    fn flatten(self) -> Self::Flat;
}

impl Sealed for HNil {}
impl<Head, Tail> Sealed for HCons<Head, Tail>
where
    Head: LayoutElement,
    Tail: Layout,
{
}

impl Layout for HNil {
    const ALIGNMENT: u32 = 1;
    const COUNT: u32 = 0;

    type Flat = HNil;
    type FromFlatBias = HNil;
    type Merge<Other: Layout> = <HNil as Merge<Other>>::Output
    where
        Self: Merge<Other>,
        <Self as Merge<Other>>::Output: Layout;

    fn from_flat(HNil: Self::FromFlatBias, _flat_layout: Self::Flat) -> Self {
        HNil
    }

    // fn unmerge_right_from_flat<Other>(
    // flat_layout: <<Other as Merge<Self>>::Output as Layout>::Flat,
    // ) -> Self
    // where
    // Other: Merge<Self>,
    // Other::Output: Layout,
    // {
    // HNil
    // }

    fn flatten(self) -> Self::Flat {
        HNil
    }
}

impl<Head, Tail> Layout for HCons<Head, Tail>
where
    Head: LayoutElement,
    Tail: Layout,
{
    const ALIGNMENT: u32 = if Head::ALIGNMENT > Tail::ALIGNMENT {
        Head::ALIGNMENT
    } else {
        Tail::ALIGNMENT
    };
    const COUNT: u32 = if Head::IS_EMPTY { 0 } else { 1 } + Tail::COUNT;

    type Flat = <Head::Flat as MaybeFlatType>::Flatten<Tail>;
    type FromFlatBias = HCons<Head::FromFlatBias, Tail::FromFlatBias>;
    type Merge<Other: Layout> = <Self as Merge<Other>>::Output
    where
        Self: Merge<Other>,
        <Self as Merge<Other>>::Output: Layout;

    fn from_flat(bias: Self::FromFlatBias, flat_layout: Self::Flat) -> Self {
        let (head, tail) = Head::unflatten_from(bias, flat_layout);

        HCons { head, tail }
    }

    // fn unmerge_right_from_flat<Other>(
    // flat_layout: <<Other as Merge<Self>>::Output as Layout>::Flat,
    // ) -> Self
    // where
    // Other: Merge<Self>,
    // Other::Output: Layout,
    // {
    // let (head, tail) =
    // HCons {
    // head: Head::Unfla,
    // tail:
    // }
    // }

    fn flatten(self) -> Self::Flat {
        self.head.flatten().flatten(self.tail)
    }
}

pub trait FlatLayout: Default + Layout<Flat = Self> {}

impl FlatLayout for HNil {}

impl<Head, Tail> FlatLayout for HCons<Head, Tail>
where
    Head: FlatType,
    Tail: FlatLayout,
{
}
