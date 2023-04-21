use {
    crate::{
        flat_type::FlatType, layout::FlatLayout, maybe_flat_type::MaybeFlatType, GuestMemory,
        GuestPointer, Layout, WitStore,
    },
    frunk::{HList, HNil},
};

pub trait FunctionResult: WitStore {
    type ExtraParameter: FlatLayout;
    type Output: MaybeFlatType;
}

impl<AllTypes> FunctionResult for AllTypes
where
    AllTypes: WitStore,
    <AllTypes::Layout as Layout>::Flat: FlatResult,
{
    type ExtraParameter = <<AllTypes::Layout as Layout>::Flat as FlatResult>::ExtraParameter;
    type Output = <<AllTypes::Layout as Layout>::Flat as FlatResult>::Output;
}

pub trait FlatResult {
    type ExtraParameter: FlatLayout;
    type Output: MaybeFlatType;
}

impl FlatResult for HNil {
    type ExtraParameter = HNil;
    type Output = ();
}

impl<AnyFlatType> FlatResult for HList![AnyFlatType]
where
    AnyFlatType: FlatType,
{
    type ExtraParameter = HNil;
    type Output = AnyFlatType;
}

impl<FirstFlatType, SecondFlatType, FlatLayoutTail> FlatResult for HList![FirstFlatType, SecondFlatType, ...FlatLayoutTail]
where
    FirstFlatType: FlatType,
    SecondFlatType: FlatType,
    FlatLayoutTail: FlatLayout,
{
    type ExtraParameter = HList![i32];
    type Output = ();
}

pub trait ResultStorage {
    type OutputFor<Results>: FlatLayout
    where
        Results: WitStore;

    fn lower_result<Results, Memory>(
        self,
        result: Results,
        memory: &mut Memory,
    ) -> Result<Self::OutputFor<Results>, Memory::Error>
    where
        Results: WitStore,
        Memory: GuestMemory;
}

impl ResultStorage for () {
    type OutputFor<Results> = <Results::Layout as Layout>::Flat
    where
        Results: WitStore;

    fn lower_result<Results, Memory>(
        self,
        result: Results,
        memory: &mut Memory,
    ) -> Result<Self::OutputFor<Results>, Memory::Error>
    where
        Results: WitStore,
        Memory: GuestMemory,
    {
        result.lower(memory)
    }
}

impl ResultStorage for GuestPointer {
    type OutputFor<Results> = HNil
    where
        Results: WitStore;

    fn lower_result<Results, Memory>(
        self,
        result: Results,
        memory: &mut Memory,
    ) -> Result<Self::OutputFor<Results>, Memory::Error>
    where
        Results: WitStore,
        Memory: GuestMemory,
    {
        result.store(memory, self)?;

        Ok(HNil)
    }
}
