use witty::{Layout, WitType};

#[test]
fn simple_bool_wrapper() {
    #[derive(WitType)]
    struct MyWrapper(bool);

    assert_eq!(MyWrapper::SIZE, 1);
    assert_eq!(<MyWrapper as WitType>::Layout::ALIGNMENT, 1);
    assert_eq!(<MyWrapper as WitType>::Layout::COUNT, 1);
}

#[test]
fn tuple_struct_without_padding() {
    #[derive(WitType)]
    struct TupleWithoutPadding(u64, i32, i16);

    assert_eq!(TupleWithoutPadding::SIZE, 14);
    assert_eq!(<TupleWithoutPadding as WitType>::Layout::ALIGNMENT, 8);
    assert_eq!(<TupleWithoutPadding as WitType>::Layout::COUNT, 3);
}

#[test]
fn tuple_struct_with_padding() {
    #[derive(WitType)]
    struct TupleWithPadding(u16, u32, i64);

    assert_eq!(TupleWithPadding::SIZE, 16);
    assert_eq!(<TupleWithPadding as WitType>::Layout::ALIGNMENT, 8);
    assert_eq!(<TupleWithPadding as WitType>::Layout::COUNT, 3);
}

#[test]
fn named_struct_with_double_padding() {
    #[derive(WitType)]
    struct RecordWithDoublePadding {
        first: u16,
        second: u32,
        third: i8,
        fourth: i64,
    }

    assert_eq!(RecordWithDoublePadding::SIZE, 24);
    assert_eq!(<RecordWithDoublePadding as WitType>::Layout::ALIGNMENT, 8);
    assert_eq!(<RecordWithDoublePadding as WitType>::Layout::COUNT, 4);
}

#[test]
fn nested_types() {
    #[derive(WitType)]
    struct Leaf {
        first: bool,
        second: u128,
    }

    assert_eq!(Leaf::SIZE, 24);
    assert_eq!(<Leaf as WitType>::Layout::ALIGNMENT, 8);
    assert_eq!(<Leaf as WitType>::Layout::COUNT, 3);

    #[derive(WitType)]
    struct Branch {
        tag: u16,
        first_leaf: Leaf,
        second_leaf: Leaf,
    }

    assert_eq!(Branch::SIZE, 56);
    assert_eq!(<Branch as WitType>::Layout::ALIGNMENT, 8);
    assert_eq!(<Branch as WitType>::Layout::COUNT, 7);
}
