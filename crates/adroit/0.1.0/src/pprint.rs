use std::fmt;

use crate::{
    Block, BlockId, Exit, Expr, FieldId, FloatBin, FuncId, Function, Instr, IntBin, IntComp,
    Module, Type, TypeId, Typedef, VarId, VariantId,
};

impl fmt::Display for TypeId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "t{}", self.0)
    }
}

impl fmt::Display for FieldId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "x{}", self.0)
    }
}

impl fmt::Display for VariantId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "v{}", self.0)
    }
}

impl fmt::Display for FuncId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "f{}", self.0)
    }
}

impl fmt::Display for BlockId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "b{}", self.0)
    }
}

impl fmt::Display for VarId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "_{}", self.0)
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Type::Bool => write!(f, "bool"),
            Type::Int => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::Ref { id } => write!(f, "{id}"),
        }
    }
}

impl fmt::Display for Module {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write_module(f, self)
    }
}

fn write_module(f: &mut fmt::Formatter, Module { types, funcs }: &Module) -> fmt::Result {
    write!(f, "(module")?;
    for (i, def) in types.iter().enumerate() {
        write!(f, "\n  (type {}", TypeId(i))?;
        write_typedef(f, def)?;
        write!(f, ")")?;
    }
    for (i, func) in funcs.iter().enumerate() {
        let Function {
            dom,
            cod,
            vars: _,
            blocks,
            entry,
        } = func;
        write!(f, "\n  (func {}", FuncId(i))?;
        for t in dom {
            write!(f, " (param {t})")?;
        }
        for t in cod {
            write!(f, " (result {t})")?;
        }
        write!(f, " (entry {})", entry)?;
        for (
            j,
            Block {
                params,
                instrs,
                exit,
            },
        ) in blocks.iter().enumerate()
        {
            write!(f, "\n    (block {}", BlockId(j))?;
            for &x in params {
                write!(f, " (param {x} {})", func.var_type(x))?;
            }
            for instr in instrs {
                write!(f, "\n      ")?;
                write_instr(f, func, instr)?;
            }
            write!(f, "\n      ")?;
            write_exit(f, exit)?;
            write!(f, ")")?;
        }
        write!(f, ")")?;
    }
    writeln!(f, ")")?;
    Ok(())
}

fn write_typedef(f: &mut fmt::Formatter, def: &Typedef) -> fmt::Result {
    match def {
        Typedef::Array { element } => {
            write!(f, " (array {element})")?;
        }
        Typedef::Struct { fields } => {
            write!(f, " (struct")?;
            for (i, field) in fields.iter().enumerate() {
                write!(f, " (field {} {field})", FieldId(i))?;
            }
            write!(f, ")")?;
        }
        Typedef::Enum { variants } => {
            write!(f, "\n    (enum")?;
            for (i, variant) in variants.iter().enumerate() {
                write!(f, "\n      (variant {} {variant})", VariantId(i))?;
            }
            write!(f, ")")?;
        }
    }
    Ok(())
}

fn write_instr(
    f: &mut fmt::Formatter,
    func: &Function,
    Instr { vars, expr }: &Instr,
) -> fmt::Result {
    write!(f, "(let ")?;
    for &x in vars {
        write!(f, "(var {x} {}) ", func.var_type(x))?;
    }
    match expr {
        Expr::IntConst { val } => write!(f, "(int.const {val})")?,
        Expr::IntComp { op, x, y } => write!(
            f,
            "(int.{} {x} {y})",
            match op {
                IntComp::Lt => "lt",
            }
        )?,
        Expr::IntBin { op, x, y } => write!(
            f,
            "(int.{} {x} {y})",
            match op {
                IntBin::Add => "add",
            }
        )?,
        Expr::FloatConst { val } => write!(f, "(float.const {val})")?,
        Expr::FloatBin { op, x, y } => write!(
            f,
            "(float.{} {x} {y})",
            match op {
                FloatBin::Add => "add",
                FloatBin::Sub => "sub",
            }
        )?,
        Expr::ArrayLen { id, var } => write!(f, "(array.len {id} {var})")?,
        Expr::ArrayGet { id, var, index } => write!(f, "(array.get {id} {var} {index})")?,
        Expr::StructMake { id, fields } => {
            write!(f, "(struct.make {id}")?;
            for x in fields {
                write!(f, " {x}")?;
            }
            write!(f, ")")?;
        }
        Expr::StructGet { id, field, var } => write!(f, "(struct.get {id} {field} {var})")?,
        Expr::EnumMake { id, variant, inner } => write!(f, "(enum.make {id} {variant} {inner})")?,
        Expr::Call { func, args } => {
            write!(f, "(call {func}")?;
            for a in args {
                write!(f, " {a}")?;
            }
            write!(f, ")")?;
        }
    }
    write!(f, ")")?;
    Ok(())
}

fn write_exit(f: &mut fmt::Formatter, exit: &Exit) -> fmt::Result {
    match exit {
        Exit::Jump { to, values } => {
            write!(f, "(jump {to}")?;
            for x in values {
                write!(f, " {x}")?;
            }
            write!(f, ")")?;
        }
        Exit::If {
            cond,
            true_to,
            true_values,
            false_to,
            false_values,
        } => {
            write!(f, "(if {cond}\n        (then {true_to}")?;
            for x in true_values {
                write!(f, " {x}")?;
            }
            write!(f, ")\n        (else {false_to}")?;
            for x in false_values {
                write!(f, " {x}")?;
            }
            write!(f, "))")?;
        }
        Exit::Match { on, cases } => {
            write!(f, "(match {on}")?;
            for block in cases {
                write!(f, "\n        (case {block})")?;
            }
            write!(f, ")")?;
        }
        Exit::Return { values } => {
            write!(f, "(return")?;
            for x in values {
                write!(f, " {x}")?;
            }
            write!(f, ")")?;
        }
    }
    Ok(())
}
