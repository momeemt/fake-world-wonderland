#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    If {
        cond: Box<Expression>,
        then: Box<Statement>,
        els: Box<Statement>,
    },
    While {
        cond: Box<Expression>,
        stmt: Box<Statement>,
    },
    Assign {
        name: String,
        expr: Box<Expression>,
    },
    Sequence {
        stmts: Vec<Box<Statement>>,
    },
    FuncDef {
        params: Vec<String>,
        body: Box<Statement>,
    }
}

