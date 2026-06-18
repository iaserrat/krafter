use super::Rng;

const BASE_OPS: usize = 15;
const DICT_OPS: usize = 2;

pub(super) enum HavocOp {
    FlipBit,
    SetI8,
    SetI16,
    SetI32,
    SubByte,
    AddByte,
    SubWord,
    AddWord,
    SubDword,
    AddDword,
    XorByte,
    DeleteBlock,
    InsertBlock,
    OverwriteBlock,
    DictOverwrite,
    DictInsert,
}

impl HavocOp {
    pub(super) fn pick(rng: &mut Rng, has_dict: bool) -> Self {
        let op_count = BASE_OPS + usize::from(has_dict) * DICT_OPS;
        match rng.below(op_count) {
            0 => Self::FlipBit,
            1 => Self::SetI8,
            2 => Self::SetI16,
            3 => Self::SetI32,
            4 => Self::SubByte,
            5 => Self::AddByte,
            6 => Self::SubWord,
            7 => Self::AddWord,
            8 => Self::SubDword,
            9 => Self::AddDword,
            10 => Self::XorByte,
            11 | 12 => Self::DeleteBlock,
            13 => Self::InsertBlock,
            14 => Self::OverwriteBlock,
            15 => Self::DictOverwrite,
            _ => Self::DictInsert,
        }
    }
}
