mod parse;
mod pprint;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TypeId(pub usize);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FieldId(pub usize);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct VariantId(pub usize);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FuncId(pub usize);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BlockId(pub usize);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct VarId(pub usize);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Type {
    Bool,
    Int,
    Float,
    Ref { id: TypeId },
}

#[derive(Clone, Debug)]
pub struct Module {
    pub types: Vec<Typedef>,
    pub funcs: Vec<Function>,
}

#[derive(Clone, Debug)]
pub enum Typedef {
    Array { element: Type },
    Struct { fields: Vec<Type> },
    Enum { variants: Vec<Type> },
}

#[derive(Clone, Debug)]
pub struct Function {
    pub dom: Vec<Type>,
    pub cod: Vec<Type>,
    pub vars: Vec<Type>,
    pub blocks: Vec<Block>,
    pub entry: BlockId,
}

impl Function {
    pub fn var_type(&self, var: VarId) -> Type {
        self.vars[var.0]
    }
}

#[derive(Clone, Debug)]
pub struct Block {
    pub params: Vec<VarId>,
    pub instrs: Vec<Instr>,
    pub exit: Exit,
}

#[derive(Clone, Debug)]
pub struct Instr {
    pub vars: Vec<VarId>,
    pub expr: Expr,
}

#[derive(Clone, Debug)]
pub enum Expr {
    IntConst {
        val: i32,
    },
    IntComp {
        op: IntComp,
        x: VarId,
        y: VarId,
    },
    IntBin {
        op: IntBin,
        x: VarId,
        y: VarId,
    },
    FloatConst {
        val: f64,
    },
    FloatBin {
        op: FloatBin,
        x: VarId,
        y: VarId,
    },
    ArrayLen {
        id: TypeId,
        var: VarId,
    },
    ArrayGet {
        id: TypeId,
        var: VarId,
        index: VarId,
    },
    StructMake {
        id: TypeId,
        fields: Vec<VarId>,
    },
    StructGet {
        id: TypeId,
        field: FieldId,
        var: VarId,
    },
    EnumMake {
        id: TypeId,
        variant: VariantId,
        inner: VarId,
    },
    Call {
        func: FuncId,
        args: Vec<VarId>,
    },
}

#[derive(Clone, Debug)]
pub enum IntComp {
    Lt,
}

#[derive(Clone, Debug)]
pub enum IntBin {
    Add,
}

#[derive(Clone, Debug)]
pub enum FloatBin {
    Add,
    Sub,
}

#[derive(Clone, Debug)]
pub enum Exit {
    Jump {
        to: BlockId,
        values: Vec<VarId>,
    },
    If {
        cond: VarId,
        true_to: BlockId,
        true_values: Vec<VarId>,
        false_to: BlockId,
        false_values: Vec<VarId>,
    },
    Match {
        on: VarId,
        cases: Vec<BlockId>,
    },
    Return {
        values: Vec<VarId>,
    },
}

pub use parse::ParseError;
