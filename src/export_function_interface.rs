use {
    crate::{
        flat_type::FlatType, layout::FlatLayout, results::ResultStorage, util::Split, GuestMemory,
        GuestPointer, Layout, WitLoad, WitStore, WitType,
    },
    frunk::HList,
    std::ops::Add,
};

pub trait ExportFunctionInterface {
    type Parameters: WitType;
    type Results: WitStore;
    type FlatInterface: FlatExportFunctionInterface<
        FlatParameters = <<Self::Parameters as WitType>::Layout as Layout>::Flat,
        ResultStorage = Self::ResultStorage,
    >;
    type Input;
    type Output;
    type ResultStorage: ResultStorage<OutputFor<Self::Results> = Self::Output>;

    fn lift_from_input<Memory>(
        input: Self::Input,
        memory: &Memory,
    ) -> Result<(Self::Parameters, Self::ResultStorage), Memory::Error>
    where
        Memory: GuestMemory;

    fn lower_results<Memory>(
        results: Self::Results,
        result_storage: Self::ResultStorage,
        memory: &mut Memory,
    ) -> Result<Self::Output, Memory::Error>
    where
        Memory: GuestMemory;
}

impl<Parameters, Results> ExportFunctionInterface for (Parameters, Results)
where
    Parameters: WitLoad,
    Results: WitStore,
    HList![
        <Parameters::Layout as Layout>::Flat,
        <Results::Layout as Layout>::Flat,
    ]: FlatExportFunctionInterface<FlatParameters = <Parameters::Layout as Layout>::Flat>,
    <() as WitType>::Layout: Layout<Flat = frunk::HNil>,
{
    type Parameters = Parameters;
    type Results = Results;
    type FlatInterface = HList![
        <Parameters::Layout as Layout>::Flat,
        <Results::Layout as Layout>::Flat,
    ];
    type Input = <Self::FlatInterface as FlatExportFunctionInterface>::Input;
    type Output = <<HList![
        <Parameters::Layout as Layout>::Flat,
        <Results::Layout as Layout>::Flat,
    ] as FlatExportFunctionInterface>::ResultStorage as ResultStorage>::OutputFor<Self::Results>;
    type ResultStorage = <HList![
        <Parameters::Layout as Layout>::Flat,
        <Results::Layout as Layout>::Flat,
    ] as FlatExportFunctionInterface>::ResultStorage;

    fn lift_from_input<Memory>(
        input: Self::Input,
        memory: &Memory,
    ) -> Result<(Self::Parameters, Self::ResultStorage), Memory::Error>
    where
        Memory: GuestMemory,
    {
        Self::FlatInterface::lift_from_input(input, memory)
    }

    fn lower_results<Memory>(
        results: Self::Results,
        result_storage: Self::ResultStorage,
        memory: &mut Memory,
    ) -> Result<Self::Output, Memory::Error>
    where
        Memory: GuestMemory,
    {
        result_storage.lower_result(results, memory)
    }
}

pub trait FlatExportFunctionInterface {
    type FlatParameters: FlatLayout;
    type Input: FlatLayout;
    type ResultStorage: ResultStorage;

    fn lift_from_input<Memory, Parameters>(
        input: Self::Input,
        memory: &Memory,
    ) -> Result<(Parameters, Self::ResultStorage), Memory::Error>
    where
        Parameters: WitLoad,
        Parameters::Layout: Layout<Flat = Self::FlatParameters>,
        Memory: GuestMemory;
}

macro_rules! direct_interface {
    ($( $types:ident ),* $(,)*) => { direct_interface!(| $( $types ),*); };

    ($( $types:ident ),* |) => { direct_interface!(@generate $( $types ),*); };

    ($( $types:ident ),* | $next_type:ident $(, $queued_types:ident )*) => {
        direct_interface!(@generate $( $types ),*);
        direct_interface!($( $types, )* $next_type | $( $queued_types ),*);
    };

    (@generate $( $types:ident ),*) => {
        direct_interface!(@generate $( $types ),* =>);
        direct_interface!(@generate $( $types ),* => FlatResult);
    };

    (@generate $( $types:ident ),* => $( $flat_result:ident )?) => {
        impl<$( $types, )* $( $flat_result )*> FlatExportFunctionInterface
            for HList![HList![$( $types, )*], HList![$( $flat_result )*]]
        where
            HList![$( $types, )*]: FlatLayout,
            $( $flat_result: FlatType, )*
        {
            type FlatParameters = HList![$( $types, )*];
            type Input = HList![$( $types, )*];
            type ResultStorage = ();

            fn lift_from_input<Memory, Parameters>(
                input: Self::Input,
                memory: &Memory,
            ) -> Result<(Parameters, Self::ResultStorage), Memory::Error>
            where
                Parameters: WitLoad,
                Parameters::Layout: Layout<Flat = Self::FlatParameters>,
                Memory: GuestMemory,
            {
                let parameters = Parameters::lift_from(input, memory)?;

                Ok((parameters, ()))
            }
        }
    };
}

direct_interface!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);

macro_rules! indirect_results {
    ($( $types:ident ),* $(,)*) => { indirect_results!(| $( $types ),*); };

    ($( $types:ident ),* |) => { indirect_results!(@generate $( $types ),*); };

    ($( $types:ident ),* | $next_type:ident $(, $queued_types:ident )*) => {
        indirect_results!(@generate $( $types ),*);
        indirect_results!($( $types, )* $next_type | $( $queued_types ),*);
    };

    (@generate $( $types:ident ),*) => {
        impl<$( $types, )* Y, Z, Tail> FlatExportFunctionInterface
            for HList![HList![$( $types, )*], HList![Y, Z, ...Tail]]
        where
            HList![$( $types, )*]: FlatLayout + Add<HList![i32]>,
            <HList![$( $types, )*] as Add<HList![i32]>>::Output:
                FlatLayout + Split<HList![$( $types, )*], Remainder = HList![i32]>,
        {
            type FlatParameters = HList![$( $types, )*];
            type Input = <Self::FlatParameters as Add<HList![i32]>>::Output;
            type ResultStorage = GuestPointer;

            fn lift_from_input<Memory, Parameters>(
                input: Self::Input,
                memory: &Memory,
            ) -> Result<(Parameters, Self::ResultStorage), Memory::Error>
            where
                Parameters: WitLoad,
                Parameters::Layout: Layout<Flat = Self::FlatParameters>,
                Memory: GuestMemory,
            {
                let (parameters_layout, result_storage_layout) = input.split();
                let parameters = Parameters::lift_from(parameters_layout, memory)?;
                let result_storage = Self::ResultStorage::lift_from(result_storage_layout, memory)?;

                Ok((parameters, result_storage))
            }
        }
    };
}

indirect_results!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);

macro_rules! indirect_parameters {
    (=> $( $flat_result:ident )? ) => {
        impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, Tail $(, $flat_result )*>
            FlatExportFunctionInterface
            for HList![
                HList![A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, ...Tail],
                HList![$( $flat_result )*],
            ]
        where
            HList![A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, ...Tail]: FlatLayout,
            $( $flat_result: FlatType, )*
        {
            type FlatParameters = HList![A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, ...Tail];
            type Input = HList![i32];
            type ResultStorage = ();

            fn lift_from_input<Memory, Parameters>(
                input: Self::Input,
                memory: &Memory,
            ) -> Result<(Parameters, Self::ResultStorage), Memory::Error>
            where
                Parameters: WitLoad,
                Parameters::Layout: Layout<Flat = Self::FlatParameters>,
                Memory: GuestMemory,
            {
                let parameters_location = GuestPointer::lift_from(input, memory)?;
                let parameters = Parameters::load(memory, parameters_location)?;

                Ok((parameters, ()))
            }
        }
    };
}

indirect_parameters!(=>);
indirect_parameters!(=> Z);

impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, OtherParameters, Y, Z, OtherResults>
    FlatExportFunctionInterface
    for HList![
        HList![A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, ...OtherParameters],
        HList![Y, Z, ...OtherResults],
    ]
where
    HList![A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, ...OtherParameters]: FlatLayout,
    HList![Y, Z, ...OtherResults]: FlatLayout,
{
    type FlatParameters =
        HList![A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, ...OtherParameters];
    type Input = HList![i32, i32];
    type ResultStorage = GuestPointer;

    fn lift_from_input<Memory, Parameters>(
        input: Self::Input,
        memory: &Memory,
    ) -> Result<(Parameters, Self::ResultStorage), Memory::Error>
    where
        Parameters: WitLoad,
        Parameters::Layout: Layout<Flat = Self::FlatParameters>,
        Memory: GuestMemory,
    {
        let (parameters_layout, result_storage_layout) = input.split();
        let parameters_location = GuestPointer::lift_from(parameters_layout, memory)?;
        let parameters = Parameters::load(memory, parameters_location)?;
        let result_storage = Self::ResultStorage::lift_from(result_storage_layout, memory)?;

        Ok((parameters, result_storage))
    }
}
