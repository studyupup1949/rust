use ast;

#[derive(Ast)]
#[phase(Parse)]
#[phase(Desugar)]
#[phase(Codegen)]
enum Expression {
    Add(Box<Self>, Box<Self>),
    Neg(Box<Self>),

    #[phase()]
    Sub(Box<Self>, Box<Self>),
}



fn main() {

}