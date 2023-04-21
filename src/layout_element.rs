use {
    crate::{
        join_flat_types::JoinFlatTypes, maybe_flat_type::MaybeFlatType, simple_type::SimpleType,
        Layout,
    },
    either::Either,
    frunk::{hlist, HCons, HNil},
};

pub trait Sealed {}

pub trait LayoutElement: Sealed + Sized {
    const ALIGNMENT: u32;
    const IS_EMPTY: bool;

    type Flat: MaybeFlatType;
    type FromFlatBias: Clone + Default;

    fn default() -> Self;
    fn flatten(self) -> Self::Flat;

    fn unflatten_from<Tail>(
        bias: HCons<Self::FromFlatBias, Tail::FromFlatBias>,
        flat_layout: <Self::Flat as MaybeFlatType>::Flatten<Tail>,
    ) -> (Self, Tail)
    where
        Tail: Layout;

    // fn unmerge_right_from_flat<Tail>(flat_layout: <S) ->
}

impl Sealed for () {}
impl<T> Sealed for T where T: SimpleType {}

impl LayoutElement for () {
    const ALIGNMENT: u32 = 1;
    const IS_EMPTY: bool = true;

    type Flat = ();
    type FromFlatBias = ();

    fn default() -> Self {}
    fn flatten(self) -> Self::Flat {}

    fn unflatten_from<Tail>(
        bias: HCons<Self::FromFlatBias, Tail::FromFlatBias>,
        flat_layout: <Self::Flat as MaybeFlatType>::Flatten<Tail>,
    ) -> (Self, Tail)
    where
        Tail: Layout,
    {
        <() as MaybeFlatType>::unflatten(bias.tail, flat_layout)
    }
}

impl<T> LayoutElement for T
where
    T: SimpleType,
{
    const ALIGNMENT: u32 = <T as SimpleType>::ALIGNMENT;
    const IS_EMPTY: bool = false;

    type Flat = <T as SimpleType>::Flat;
    type FromFlatBias = ();

    fn default() -> Self {
        Default::default()
    }

    fn flatten(self) -> Self::Flat {
        <T as SimpleType>::flatten(self)
    }

    fn unflatten_from<Tail>(
        bias: HCons<Self::FromFlatBias, Tail::FromFlatBias>,
        flat_layout: <Self::Flat as MaybeFlatType>::Flatten<Tail>,
    ) -> (Self, Tail)
    where
        Tail: Layout,
    {
        let (flat_type, tail) = <Self::Flat as MaybeFlatType>::unflatten(bias.tail, flat_layout);
        (<T as SimpleType>::unflatten_from(flat_type), tail)
    }
}

// impl<T> Sealed for Either<T, ()> where T: LayoutElement {}

// impl<T> LayoutElement for Either<T, ()>
// where
// T: LayoutElement,
// {
// type Flat = T::Flat;

// fn flatten(self) -> Self::Flat {
// match self {
// Either::Left(value) => value.flatten(),
// Either::Right(()) => T::default(),
// }
// }
// }

impl<L, R> Sealed for Either<L, R>
where
    L: LayoutElement,
    R: LayoutElement,
{
}

impl<R> LayoutElement for Either<(), R>
where
    R: LayoutElement,
{
    const ALIGNMENT: u32 = R::ALIGNMENT;
    const IS_EMPTY: bool = R::IS_EMPTY;

    type Flat = R::Flat;
    type FromFlatBias = Option<Either<(), R::FromFlatBias>>;

    fn default() -> Self {
        Either::Right(R::default())
    }

    fn flatten(self) -> Self::Flat {
        match self {
            Either::Left(()) => R::default().flatten(),
            Either::Right(value) => value.flatten(),
        }
    }

    fn unflatten_from<Tail>(
        bias: HCons<Self::FromFlatBias, Tail::FromFlatBias>,
        flat_layout: <Self::Flat as MaybeFlatType>::Flatten<Tail>,
    ) -> (Self, Tail)
    where
        Tail: Layout,
    {
        match bias.head {
            Some(Either::Left(())) => {
                let (_element, tail) =
                    <Self::Flat as MaybeFlatType>::unflatten(bias.tail, flat_layout);
                (Either::Left(()), tail)
            }
            Some(Either::Right(right_bias)) => {
                let (element, tail) =
                    R::unflatten_from(hlist![right_bias, ...bias.tail], flat_layout);
                (Either::Right(element), tail)
            }
            None => {
                let right_bias = Default::default();
                let (element, tail) =
                    R::unflatten_from(hlist![right_bias, ...bias.tail], flat_layout);
                (Either::Right(element), tail)
            }
        }
    }
}

impl<L, R> LayoutElement for Either<L, R>
where
    L: SimpleType,
    R: LayoutElement,
    Either<L::Flat, R::Flat>: JoinFlatTypes,
{
    const ALIGNMENT: u32 = if L::ALIGNMENT > R::ALIGNMENT {
        L::ALIGNMENT
    } else {
        R::ALIGNMENT
    };
    const IS_EMPTY: bool = L::IS_EMPTY && R::IS_EMPTY;

    type Flat = <Either<L::Flat, R::Flat> as JoinFlatTypes>::Flat;
    type FromFlatBias = Option<Either<(), R::FromFlatBias>>;

    fn default() -> Self {
        Either::Right(R::default())
    }

    fn flatten(self) -> Self::Flat {
        match self {
            Either::Left(left) => Either::Left(left.flatten()).join(),
            Either::Right(right) => Either::Right(right.flatten()).join(),
        }
    }

    fn unflatten_from<Tail>(
        bias: HCons<Self::FromFlatBias, Tail::FromFlatBias>,
        flat_layout: <Self::Flat as MaybeFlatType>::Flatten<Tail>,
    ) -> (Self, Tail)
    where
        Tail: Layout,
    {
        let (flat_type, tail) = <Self::Flat as MaybeFlatType>::unflatten(bias.tail, flat_layout);

        match bias.head {
            Some(Either::Left(())) | None => {
                let element = L::unflatten_from(flat_type.split_into());

                (Either::Left(element), tail)
            }
            Some(Either::Right(right_bias)) => {
                let flat_element = flat_type.split_into::<R::Flat>();
                let (element, _) =
                    R::unflatten_from(hlist![right_bias], flat_element.flatten(HNil));

                (Either::Right(element), tail)
            }
        }
    }
}
