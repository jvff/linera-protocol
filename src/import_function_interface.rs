use {
    crate::{
        flat_type::FlatType, layout::FlatLayout, GuestMemory, GuestPointer, Layout, WitLoad,
        WitStore, WitType,
    },
    frunk::HList,
};

pub trait ImportFunctionInterface {
    type Parameters: WitStore;
    type Results: WitLoad;
    type Input: FlatLayout;
    type Output: FlatLayout;

    fn lower_parameters<Memory>(
        parameters: Self::Parameters,
        memory: &mut Memory,
    ) -> Result<Self::Input, Memory::Error>
    where
        Memory: GuestMemory;

    fn lift_from_output<Memory>(
        output: Self::Output,
        memory: &Memory,
    ) -> Result<Self::Results, Memory::Error>
    where
        Memory: GuestMemory;
}

impl<Parameters, Results> ImportFunctionInterface for (Parameters, Results)
where
    Parameters: WitStore,
    Results: WitLoad,
    <Parameters::Layout as Layout>::Flat: FlatParameters,
    <Results::Layout as Layout>::Flat: FlatResults,
{
    type Parameters = Parameters;
    type Results = Results;
    type Input = <<Parameters::Layout as Layout>::Flat as FlatParameters>::Input;
    type Output = <<Results::Layout as Layout>::Flat as FlatResults>::Output;

    fn lower_parameters<Memory>(
        parameters: Self::Parameters,
        memory: &mut Memory,
    ) -> Result<Self::Input, Memory::Error>
    where
        Memory: GuestMemory,
    {
        <<Parameters::Layout as Layout>::Flat as FlatParameters>::lower_parameters(
            parameters, memory,
        )
    }

    fn lift_from_output<Memory>(
        output: Self::Output,
        memory: &Memory,
    ) -> Result<Self::Results, Memory::Error>
    where
        Memory: GuestMemory,
    {
        <<Results::Layout as Layout>::Flat as FlatResults>::lift_from_output(output, memory)
    }
}

pub trait FlatParameters: FlatLayout {
    type Input: FlatLayout;

    fn lower_parameters<Parameters, Memory>(
        parameters: Parameters,
        memory: &mut Memory,
    ) -> Result<Self::Input, Memory::Error>
    where
        Parameters: WitStore,
        <Parameters as WitType>::Layout: Layout<Flat = Self>,
        Memory: GuestMemory;
}

macro_rules! direct_parameters {
    ($( $types:ident ),* $(,)*) => { direct_parameters!(| $( $types ),*); };

    ($( $types:ident ),* |) => { direct_parameters!(@generate $( $types ),*); };

    ($( $types:ident ),* | $next_type:ident $(, $queued_types:ident )*) => {
        direct_parameters!(@generate $( $types ),*);
        direct_parameters!($( $types, )* $next_type | $( $queued_types ),*);
    };

    (@generate $( $types:ident ),*) => {
        impl<$( $types ),*> FlatParameters for HList![$( $types ),*]
        where
            $( $types: FlatType, )*
        {
            type Input = HList![$( $types ),*];

            fn lower_parameters<Parameters, Memory>(
                parameters: Parameters,
                memory: &mut Memory,
            ) -> Result<Self::Input, Memory::Error>
            where
                Parameters: WitStore,
                <Parameters as WitType>::Layout: Layout<Flat = Self>,
                Memory: GuestMemory,
            {
                parameters.lower(memory)
            }
        }
    };
}

direct_parameters!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);

impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, Tail> FlatParameters for HList![A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, ...Tail]
where
    A: FlatType,
    B: FlatType,
    C: FlatType,
    D: FlatType,
    E: FlatType,
    F: FlatType,
    G: FlatType,
    H: FlatType,
    I: FlatType,
    J: FlatType,
    K: FlatType,
    L: FlatType,
    M: FlatType,
    N: FlatType,
    O: FlatType,
    P: FlatType,
    Q: FlatType,
    Tail: FlatLayout,
{
    type Input = HList![i32];

    fn lower_parameters<Parameters, Memory>(
        parameters: Parameters,
        memory: &mut Memory,
    ) -> Result<Self::Input, Memory::Error>
    where
        Parameters: WitStore,
        <Parameters as WitType>::Layout: Layout<Flat = Self>,
        Memory: GuestMemory,
    {
        let location = memory.allocate(Parameters::SIZE)?;

        parameters.store(memory, *location)?;
        location.lower(memory)
    }
}

pub trait FlatResults: FlatLayout {
    type Output: FlatLayout;

    fn lift_from_output<Results, Memory>(
        output: Self::Output,
        memory: &Memory,
    ) -> Result<Results, Memory::Error>
    where
        Results: WitLoad,
        <Results as WitType>::Layout: Layout<Flat = Self>,
        Memory: GuestMemory;
}

impl FlatResults for HList![] {
    type Output = HList![];

    fn lift_from_output<Results, Memory>(
        output: Self::Output,
        memory: &Memory,
    ) -> Result<Results, Memory::Error>
    where
        Results: WitLoad,
        <Results as WitType>::Layout: Layout<Flat = Self>,
        Memory: GuestMemory,
    {
        Results::lift_from(output, memory)
    }
}

impl<FlatResult> FlatResults for HList![FlatResult]
where
    FlatResult: FlatType,
{
    type Output = HList![FlatResult];

    fn lift_from_output<Results, Memory>(
        output: Self::Output,
        memory: &Memory,
    ) -> Result<Results, Memory::Error>
    where
        Results: WitLoad,
        <Results as WitType>::Layout: Layout<Flat = Self>,
        Memory: GuestMemory,
    {
        Results::lift_from(output, memory)
    }
}

impl<FirstResult, SecondResult, Tail> FlatResults for HList![FirstResult, SecondResult, ...Tail]
where
    FirstResult: FlatType,
    SecondResult: FlatType,
    Tail: FlatLayout,
{
    type Output = HList![i32];

    fn lift_from_output<Results, Memory>(
        output: Self::Output,
        memory: &Memory,
    ) -> Result<Results, Memory::Error>
    where
        Results: WitLoad,
        <Results as WitType>::Layout: Layout<Flat = Self>,
        Memory: GuestMemory,
    {
        let location = GuestPointer::lift_from(output, memory)?;

        Results::load(memory, location)
    }
}
