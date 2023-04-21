use {
    either::Either,
    frunk::{hlist, HCons, HNil},
    std::fmt::{self, Display, Formatter},
};

#[derive(Clone, Copy, Debug)]
pub struct ConcatDisplay<Head, Tail>(pub Head, pub Tail);

impl<Head, Tail> Display for ConcatDisplay<Head, Tail>
where
    Head: Display,
    Tail: Display,
{
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}{}", self.0, self.1)
    }
}

pub trait Split<Target> {
    type Remainder;

    fn split(self) -> (Target, Self::Remainder);
}

impl<AllTypes> Split<HNil> for AllTypes {
    type Remainder = AllTypes;

    fn split(self) -> (HNil, Self::Remainder) {
        (hlist![], self)
    }
}

impl<Head, SourceTail, TargetTail> Split<HCons<Head, TargetTail>> for HCons<Head, SourceTail>
where
    SourceTail: Split<TargetTail>,
{
    type Remainder = <SourceTail as Split<TargetTail>>::Remainder;

    fn split(self) -> (HCons<Head, TargetTail>, Self::Remainder) {
        let (tail, remainder) = self.tail.split();

        (
            HCons {
                head: self.head,
                tail,
            },
            remainder,
        )
    }
}

pub trait Unmerge {
    type Left;
    type Right;

    fn unmerge_left(self) -> Self::Left;
    fn unmerge_right(self) -> Self::Right;
}

impl Unmerge for HNil {
    type Left = HNil;
    type Right = HNil;

    fn unmerge_left(self) -> Self::Left {
        HNil
    }

    fn unmerge_right(self) -> Self::Right {
        HNil
    }
}

impl<LeftHead, RightHead, Tail> Unmerge for HCons<Either<LeftHead, RightHead>, Tail>
where
    Tail: Unmerge,
{
    type Left = HCons<LeftHead, <Tail as Unmerge>::Left>;
    type Right = HCons<RightHead, <Tail as Unmerge>::Right>;

    fn unmerge_left(self) -> Self::Left {
        let Either::Left(head) = self.head
            else { panic!("Incorrect unmerging of heterogeneous list"); };

        HCons {
            head,
            tail: self.tail.unmerge_left(),
        }
    }

    fn unmerge_right(self) -> Self::Right {
        let Either::Right(head) = self.head
            else { panic!("Incorrect unmerging of heterogeneous list"); };

        HCons {
            head,
            tail: self.tail.unmerge_right(),
        }
    }
}

pub trait Merge<Other>: Sized {
    type Output: Unmerge;

    fn merge_left(self) -> Self::Output;
    fn unmerge_from(merged: Self::Output) -> Self;
}

impl Merge<HNil> for HNil {
    type Output = HNil;

    fn merge_left(self) -> Self::Output {
        HNil
    }

    fn unmerge_from(_merged: Self::Output) -> Self {
        HNil
    }
}

impl<Head, Tail> Merge<HNil> for HCons<Head, Tail>
where
    Tail: Merge<HNil>,
{
    type Output = HCons<Either<Head, ()>, <Tail as Merge<HNil>>::Output>;

    fn merge_left(self) -> Self::Output {
        HCons {
            head: Either::Left(self.head),
            tail: self.tail.merge_left(),
        }
    }

    fn unmerge_from(merged: Self::Output) -> Self {
        let Either::Left(head) = merged.head
            else { panic!("Incorrect unmerging of heterogeneous list"); };

        HCons {
            head,
            tail: Merge::unmerge_from(merged.tail),
        }
    }
}

impl<Head, Tail> Merge<HCons<Head, Tail>> for HNil
where
    HNil: Merge<Tail>,
{
    type Output = HCons<Either<(), Head>, <HNil as Merge<Tail>>::Output>;

    fn merge_left(self) -> Self::Output {
        HCons {
            head: Either::Left(()),
            tail: self.merge_left(),
        }
    }

    fn unmerge_from(_merged: Self::Output) -> Self {
        HNil
    }
}

impl<LeftHead, LeftTail, RightHead, RightTail> Merge<HCons<RightHead, RightTail>>
    for HCons<LeftHead, LeftTail>
where
    LeftTail: Merge<RightTail>,
{
    type Output = HCons<Either<LeftHead, RightHead>, <LeftTail as Merge<RightTail>>::Output>;

    fn merge_left(self) -> Self::Output {
        HCons {
            head: Either::Left(self.head),
            tail: self.tail.merge_left(),
        }
    }

    fn unmerge_from(merged: Self::Output) -> Self {
        let Either::Left(head) =merged.head
            else { panic!("Incorrect unmerging of heterogeneous list"); };

        HCons {
            head,
            tail: Merge::unmerge_from(merged.tail),
        }
    }
}

pub trait BuildByRepeating<Element>
where
    Element: Clone,
{
    fn build_by_repeating(element: &Element) -> Self;
}

impl<AnyType> BuildByRepeating<AnyType> for HNil
where
    AnyType: Clone,
{
    fn build_by_repeating(_element: &AnyType) -> Self {
        HNil
    }
}

impl<Head, Tail> BuildByRepeating<Head> for HCons<Head, Tail>
where
    Head: Clone,
    Tail: BuildByRepeating<Head>,
{
    fn build_by_repeating(element: &Head) -> Self {
        HCons {
            head: element.clone(),
            tail: Tail::build_by_repeating(element),
        }
    }
}

pub trait RepeatToFill: Clone {
    fn repeat_to_fill<Target: BuildByRepeating<Self>>(&self) -> Target;
}

impl<AllTypes> RepeatToFill for AllTypes
where
    AllTypes: Clone,
{
    fn repeat_to_fill<Target: BuildByRepeating<Self>>(&self) -> Target {
        Target::build_by_repeating(self)
    }
}
