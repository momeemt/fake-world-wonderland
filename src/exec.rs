use std::collections::HashMap;

use anyhow::Result;

use crate::ast::{Expression, Statement};

type Environment = HashMap<String, Expression>;

pub fn evaluate(expr: Expression, env: Environment) -> Result<Expression> {
    match expr {
        Expression::Var { name } => Ok(env[&name].clone()),
        Expression::BinExp { op, lhs, rhs } => {
            let left = evaluate(*lhs, env.clone())?;
            let right = evaluate(*rhs, env.clone())?;
            let left_value = if let Expression::Int { value } = left {
                value
            } else {
                anyhow::bail!("Expected to Expression::Int {:?}", left);
            };
            let right_value = if let Expression::Int { value } = right {
                value
            } else {
                anyhow::bail!("Expected to Expression::Int {:?}", right);
            };
            match op.as_str() {
                "+" => Ok(Expression::Int { value: left_value + right_value }),
                "-" => Ok(Expression::Int { value: left_value - right_value }),
                "*" => Ok(Expression::Int { value: left_value * right_value }),
                "/" => Ok(Expression::Int { value: left_value / right_value }),
                _ => anyhow::bail!("Unknown op: {}", op),
            }
        },
        Expression::Int { value } => Ok(Expression::Int { value }),
        _ => anyhow::bail!("Unknown expression: {:?}", expr),
    }
}

pub fn execute(stmt: Statement, env: Environment) -> Result<Environment> {
    match stmt {
        Statement::If { cond, then, els } => {
            let cond = evaluate(*cond, env.clone())?;
            let cond_value = if let Expression::Int { value } = cond {
                value
            } else {
                anyhow::bail!("Expected to Expression::Int {:?}", cond);
            };
            if cond_value != 0 {
                execute(*then, env)
            } else {
                execute(*els, env)
            }
        },
        Statement::While { cond, stmt } => {
            let mut current_env = env.clone();
            while let Expression::Int { value } = evaluate(*cond.clone(), current_env.clone())? {
                if value == 0 {
                    break
                }
                current_env = execute((*stmt).clone(), current_env.clone())?;
            }
            Ok(current_env)
        },
        Statement::Assign { name, expr } => {
            let value = evaluate(*expr, env.clone())?;
            let mut current_env = env.clone();
            current_env.insert(name, value);
            Ok(current_env)
        },
        Statement::Sequence { stmts } => {
            let mut current_env = env.clone();
            for stmt in stmts {
                current_env = execute(*stmt, current_env)?;
            }
            Ok(current_env)
        },
        _ => anyhow::bail!("Unknown statement: {:?}", stmt)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use std::collections::HashMap;

    use crate::ast::{Expression, Statement};

    use super::execute;

    #[test]
    fn test_statement() -> Result<()> {
        let mut env = HashMap::new();
        env.insert(String::from("i"), Expression::Int { value: 10 });

        let mut expect_env = HashMap::new();
        expect_env.insert(String::from("i"), Expression::Int { value: 0 });

        let stmt = Statement::While {
            cond: Box::new(Expression::Var { name: String::from("i") }),
            stmt: Box::new(Statement::Assign {
                name: String::from("i"),
                expr: Box::new(Expression::BinExp {
                    op: String::from("-"),
                    lhs: Box::new(Expression::Var { name: String::from("i") }),
                    rhs: Box::new(Expression::Int { value: 1 })
                })
            })
        };
        let res_env = execute(stmt, env)?;
       
        assert_eq!(expect_env, res_env);

        Ok(())
    }

    #[test]
    fn test_statement2() -> Result<()> {
        let mut expect_env = HashMap::new();
        expect_env.insert(String::from("i"), Expression::Int { value: 0 });
        expect_env.insert(String::from("sum"), Expression::Int { value: 55 });

        let stmt = Statement::Sequence { stmts: vec![
            Box::new(Statement::Assign {
                name: String::from("i"),
                expr: Box::new(Expression::Int { value: 10 }),
            }),
            Box::new(Statement::Assign {
                name: String::from("sum"),
                expr: Box::new(Expression::Int { value: 0 }),
            }),
            Box::new(Statement::While {
                cond: Box::new(Expression::Var { name: String::from("i") }),
                stmt: Box::new(Statement::Sequence { stmts: vec![
                    Box::new(Statement::Assign {
                        name: String::from("sum"),
                        expr: Box::new(Expression::BinExp {
                            op: String::from("+"),
                            lhs: Box::new(Expression::Var { name: String::from("sum") }),
                            rhs: Box::new(Expression::Var { name: String::from("i") }),
                        }),
                    }),
                    Box::new(Statement::Assign {
                        name: String::from("i"),
                        expr: Box::new(Expression::BinExp {
                            op: String::from("-"),
                            lhs: Box::new(Expression::Var { name: String::from("i") }),
                            rhs: Box::new(Expression::Int { value: 1 }),
                        })
                    })
                ]})
            })
        ]};
        let res_env = execute(stmt, HashMap::new())?;

        assert_eq!(expect_env, res_env);

        Ok(())
    }
}
