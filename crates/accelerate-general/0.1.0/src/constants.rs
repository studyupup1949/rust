pub type RowColMajor = CblasOrder;
pub type TransposeMode = CblasTranspose;
pub type UpOrLowTriangle = CblasUpLow;
pub type IsDiagUnit = CblasDiag;
pub type MultiplyOrder = CblasSide;

#[repr(i32)]
pub enum CblasOrder {
    RowMajor = 101,
    ColMajor = 102,
}

#[repr(i32)]
pub enum CblasTranspose {
    NoTrans = 111,
    Trans = 112,
    ConjTrans = 113,
    AtlasConj = 114,
}

#[repr(i32)]
pub enum CblasUpLow {
    Upper = 121,
    Lower = 122,
}

#[repr(i32)]
pub enum CblasDiag {
    NonUnit = 131,
    Unit = 132,
}

#[repr(i32)]
pub enum CblasSide {
    Left = 141,
    Right = 142,
}
