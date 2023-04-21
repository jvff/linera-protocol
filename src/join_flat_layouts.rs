use {
    super::{flat_type::FlatType, join_flat_types::JoinFlatTypes},
    either::Either,
    frunk::{HCons, HNil},
};

pub trait JoinFlatLayouts<Target> {
    fn join(self) -> Target;
}

impl JoinFlatLayouts<HNil> for HNil {
    fn join(self) -> HNil {
        HNil
    }
}

impl<TargetHead, TargetTail> JoinFlatLayouts<HCons<TargetHead, TargetTail>> for HNil
where
    TargetHead: Default,
    HNil: JoinFlatLayouts<TargetTail>,
{
    fn join(self) -> HCons<TargetHead, TargetTail> {
        HCons {
            head: TargetHead::default(),
            tail: HNil.join(),
        }
    }
}

impl<SourceHead, SourceTail, TargetHead, TargetTail> JoinFlatLayouts<HCons<TargetHead, TargetTail>>
    for HCons<SourceHead, SourceTail>
where
    Either<SourceHead, TargetHead>: JoinFlatTypes<Flat = TargetHead>,
    SourceTail: JoinFlatLayouts<TargetTail>,
{
    fn join(self) -> HCons<TargetHead, TargetTail> {
        HCons {
            head: Either::Left(self.head).join(),
            tail: self.tail.join(),
        }
    }
}

pub trait SplitFlatLayouts<Target> {
    fn split(self) -> Target;
}

impl<AllFlatLayouts> SplitFlatLayouts<HNil> for AllFlatLayouts {
    fn split(self) -> HNil {
        HNil
    }
}

impl<Source, TargetTail> SplitFlatLayouts<HCons<(), TargetTail>> for Source
where
    Source: SplitFlatLayouts<TargetTail>,
{
    fn split(self) -> HCons<(), TargetTail> {
        HCons {
            head: (),
            tail: self.split(),
        }
    }
}

impl<SourceHead, SourceTail, TargetHead, TargetTail> SplitFlatLayouts<HCons<TargetHead, TargetTail>>
    for HCons<SourceHead, SourceTail>
where
    TargetHead: FlatType,
    SourceHead: FlatType,
    SourceTail: SplitFlatLayouts<TargetTail>,
{
    fn split(self) -> HCons<TargetHead, TargetTail> {
        HCons {
            head: self.head.split_into(),
            tail: self.tail.split(),
        }
    }
}
