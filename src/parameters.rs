use crate::{
    layout::{FlatLayout, Layout},
    results::FunctionResult,
    WitType,
};

// pub trait ShortParameters {}
// pub trait LongParameters {}
// pub trait ShortParametersWithReturnParameter {}
// pub trait LongParametersWithReturnParameter {}

// impl ShortParameters for HList![] {}
// impl ShortParameters for AllFlatTypes where AllFlatTypes: FlatType {}

// impl LongParameters for

trait FunctionParameters {
    type Input<ReturnType>: FlatLayout
    where
        ReturnType: FunctionResult;

    type LayoutWithReturnParameterFor<ReturnType>: Layout
    where
        ReturnType: FunctionResult;
}

impl<AllTypes> FunctionParameters for AllTypes
where
    AllTypes: WitType,
{
    type Input<ReturnType> =
        <<AllTypes::Layout as Layout>::Flat as FlatParameters>::Input<ReturnType>
    where
        ReturnType: FunctionResult;

    type LayoutWithReturnParameterFor<ReturnType> =
        <AllTypes::Layout as Layout>::Append<ReturnType::ExtraParameter>
            where ReturnType:FunctionResult;
}

trait FlatParameters {
    type Input<ReturnType>
    where
        ReturnType: FunctionResult;
}

impl FlatParameters for {
}
