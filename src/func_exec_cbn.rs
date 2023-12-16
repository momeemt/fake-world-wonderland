use std::collections::HashMap;

use anyhow::Result;

use crate::ast::{Expression, Statement};

type Environment = HashMap<String, Thunk>;
type FunctionEnvironment = HashMap<String, Statement>;

#[derive(Clone, Debug)]
pub struct Thunk {
    expr: Box<Expression>,
    env: Box<Environment>,
    func_env: Box<FunctionEnvironment>,
}

pub fn evaluate(
    expr: Box<Expression>,
    env: Box<Environment>,
    func_env: Box<FunctionEnvironment>,
) -> Result<Expression> {
    fn make_thunk_list(
        args: &Vec<Box<Expression>>,
        env: &Box<Environment>,
        func_env: &Box<FunctionEnvironment>,
    ) -> Result<Vec<Thunk>> {
        args.iter()
            .map(|arg| {
                Ok(Thunk {
                    expr: arg.clone(),
                    env: env.clone(),
                    func_env: func_env.clone(),
                })
            })
            .collect()
    }

    fn eval_thunk(thunk: &Thunk) -> Result<Expression> {
        evaluate(
            thunk.expr.clone(),
            thunk.env.clone(),
            thunk.func_env.clone(),
        )
    }

    fn exec_fun(
        func_name: &str,
        args: Vec<Thunk>,
        func_env: &Box<FunctionEnvironment>,
    ) -> Result<Expression> {
        fn build_environment_from_args(
            params: &Vec<String>,
            args: Vec<Thunk>,
        ) -> Result<Environment> {
            if params.len() != args.len() {
                anyhow::bail!("Wrong number of args: {:?} for {:?}", args, params);
            }
            let mut env = HashMap::new();
            for (param, arg) in params.iter().zip(args.into_iter()) {
                env.insert(param.clone(), arg);
            }
            Ok(env)
        }
        let stmt = func_env
            .get(func_name)
            .ok_or_else(|| anyhow::anyhow!("Unknown function: {}", func_name))?;
        let (params, body) = match stmt {
            Statement::FuncDef { params, body } => (params.clone(), body.clone()),
            _ => anyhow::bail!("Expected to Statement::FuncDef {:?}", stmt),
        };
        let mut env = build_environment_from_args(&params, args)?;
        let binding = Box::new(env.clone());
        env.insert(
            String::from("return"),
            Thunk {
                expr: Box::new(Expression::Int { value: 0 }),
                env: binding,
                func_env: func_env.clone(),
            },
        );
        let _ = execute(body, Box::new(env.clone()), func_env.clone())?;
        Ok(eval_thunk(env.get("return").ok_or_else(|| {
            anyhow::anyhow!("Expected to return value")
        })?)?)
    }

    match *expr {
        Expression::Var { ref name } => {
            Ok(eval_thunk(env.get(name).ok_or_else(|| {
                anyhow::anyhow!("Unknown variable: {}", name)
            })?)?)
        }
        Expression::BinExp {
            ref op,
            ref lhs,
            ref rhs,
        } => {
            let left = evaluate(lhs.clone(), env.clone(), func_env.clone())?;
            let right = evaluate(rhs.clone(), env.clone(), func_env.clone())?;
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
                "+" => Ok(Expression::Int {
                    value: left_value + right_value,
                }),
                "-" => Ok(Expression::Int {
                    value: left_value - right_value,
                }),
                "*" => Ok(Expression::Int {
                    value: left_value * right_value,
                }),
                "/" => Ok(Expression::Int {
                    value: left_value / right_value,
                }),
                ">" => Ok(Expression::Int {
                    value: if left_value > right_value { 1 } else { 0 },
                }),
                "<" => Ok(Expression::Int {
                    value: if left_value < right_value { 1 } else { 0 },
                }),
                _ => anyhow::bail!("Unknown op: {}", op),
            }
        }
        Expression::Int { value } => Ok(Expression::Int { value }),
        Expression::Call { ref name, ref args } => {
            exec_fun(name, make_thunk_list(args, &env, &func_env)?, &func_env)
        }
    }
}

pub fn execute(
    stmt: Box<Statement>,
    env: Box<Environment>,
    func_env: Box<FunctionEnvironment>,
) -> Result<Box<Environment>> {
    match *stmt {
        Statement::If { cond, then, els } => {
            let cond = evaluate(cond, env.clone(), func_env.clone())?;
            let cond_value = if let Expression::Int { value } = cond {
                value
            } else {
                anyhow::bail!("Expected to Expression::Int {:?}", cond);
            };
            if cond_value != 0 {
                execute(then, env.clone(), func_env.clone())
            } else {
                execute(els, env, func_env.clone())
            }
        }
        Statement::While { cond, stmt } => {
            let mut current_env = env.clone();
            while let Expression::Int { value } =
                evaluate(cond.clone(), current_env.clone(), func_env.clone())?
            {
                if value == 0 {
                    break;
                }
                current_env = execute(stmt.clone(), current_env.clone(), func_env.clone())?;
            }
            Ok(current_env)
        }
        Statement::Assign { name, expr } => {
            let mut env = env.clone();
            let expr = evaluate(expr, env.clone(), func_env.clone())?;
            env.insert(
                name.to_string(),
                Thunk {
                    expr: Box::new(expr),
                    env: env.clone(),
                    func_env,
                },
            );
            Ok(env)
        }
        Statement::Sequence { stmts } => {
            let mut current_env = env.clone();
            for stmt in stmts {
                current_env = execute(stmt, current_env, func_env.clone())?;
            }
            Ok(current_env)
        }
        _ => anyhow::bail!("Unknown statement: {:?}", stmt),
    }
}

pub fn define_function(
    name: &str,
    params: Vec<String>,
    body: Statement,
    func_env: &mut FunctionEnvironment,
) {
    func_env.insert(
        name.to_string(),
        Statement::FuncDef {
            params,
            body: Box::new(body),
        },
    );
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::{
        ast::{Expression, Statement},
        func_exec_cbv::evaluate,
    };

    use std::collections::HashMap;

    use super::define_function;

    #[test]
    fn test_func_exec_cbv1() -> Result<()> {
        let mut func_env = HashMap::new();
        define_function(
            "fun1",
            vec!["i".to_string()],
            Statement::Sequence {
                stmts: vec![
                    Box::new(Statement::Assign {
                        name: "return".to_string(),
                        expr: Box::new(Expression::Int { value: 0 }),
                    }),
                    Box::new(Statement::While {
                        cond: Box::new(Expression::Var {
                            name: "i".to_string(),
                        }),
                        stmt: Box::new(Statement::Sequence {
                            stmts: vec![
                                Box::new(Statement::Assign {
                                    name: "return".to_string(),
                                    expr: Box::new(Expression::BinExp {
                                        op: "+".to_string(),
                                        lhs: Box::new(Expression::Var {
                                            name: "return".to_string(),
                                        }),
                                        rhs: Box::new(Expression::Var {
                                            name: "i".to_string(),
                                        }),
                                    }),
                                }),
                                Box::new(Statement::Assign {
                                    name: "i".to_string(),
                                    expr: Box::new(Expression::BinExp {
                                        op: "-".to_string(),
                                        lhs: Box::new(Expression::Var {
                                            name: "i".to_string(),
                                        }),
                                        rhs: Box::new(Expression::Int { value: 1 }),
                                    }),
                                }),
                            ],
                        }),
                    }),
                ],
            },
            &mut func_env,
        );
        let mut env = HashMap::new();
        env.insert("i".to_string(), Expression::Int { value: 10 });
        let result = evaluate(
            &Expression::Call {
                name: "fun1".to_string(),
                args: vec![Box::new(Expression::Var {
                    name: "i".to_string(),
                })],
            },
            &env,
            &func_env,
        )?;
        assert_eq!(result, Expression::Int { value: 55 });
        Ok(())
    }

    #[test]
    fn test_func_exec_cbv2() -> Result<()> {
        let mut func_env = HashMap::new();
        define_function(
            "fun2",
            vec!["i".to_string()],
            Statement::If {
                cond: Box::new(Expression::BinExp {
                    op: "<".to_string(),
                    lhs: Box::new(Expression::Int { value: 0 }),
                    rhs: Box::new(Expression::Var {
                        name: "i".to_string(),
                    }),
                }),
                then: Box::new(Statement::Assign {
                    name: "return".to_string(),
                    expr: Box::new(Expression::BinExp {
                        op: "+".to_string(),
                        lhs: Box::new(Expression::Var {
                            name: "i".to_string(),
                        }),
                        rhs: Box::new(Expression::Call {
                            name: "fun2".to_string(),
                            args: vec![Box::new(Expression::BinExp {
                                op: "-".to_string(),
                                lhs: Box::new(Expression::Var {
                                    name: "i".to_string(),
                                }),
                                rhs: Box::new(Expression::Int { value: 1 }),
                            })],
                        }),
                    }),
                }),
                els: Box::new(Statement::Assign {
                    name: "return".to_string(),
                    expr: Box::new(Expression::Int { value: 0 }),
                }),
            },
            &mut func_env,
        );
        let mut env = HashMap::new();
        env.insert("i".to_string(), Expression::Int { value: 10 });
        let result = evaluate(
            &Expression::Call {
                name: "fun2".to_string(),
                args: vec![Box::new(Expression::Var {
                    name: "i".to_string(),
                })],
            },
            &env,
            &func_env,
        )?;
        assert_eq!(result, Expression::Int { value: 55 });
        Ok(())
    }
}
