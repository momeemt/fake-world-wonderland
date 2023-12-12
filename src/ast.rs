#[derive(Debug, Clone)]
pub enum Expression {
    BinExp {
        op: String,
        lhs: Box<Expression>,
        rhs: Box<Expression>,
    },
    Int {
        value: i32,
    },
    Var {
        name: String
    },
    Call {
        name: String,
        args: Vec<Box<Expression>>,
    },
}

pub enum Statement {
    If(IfStatement),
    While(WhileStatement),
    Assign(AssignStatement),
    Sequence(SequenceStatement),
    FuncDef(FunctionDefinition),
}

pub struct IfStatement {
    pub cond: Box<Expression>,
    pub then: Box<Statement>,
    pub els: Box<Statement>,
}

pub struct WhileStatement {
    pub cond: Box<Expression>,
    pub stmt: Box<Statement>,
}

pub struct AssignStatement {
    pub variable_name: String,
    pub expr: Box<Expression>,
}

pub struct SequenceStatement {
    pub stmts: Vec<Box<Statement>>,
}

pub struct FunctionDefinition {
    pub params: Vec<String>,
    pub body: Box<Statement>,
}

