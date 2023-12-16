use std::collections::HashMap;

use anyhow::Result;

use crate::ast::{Expression, Statement};

type Environment = HashMap<String, Expression>;
type FunctionEnvironment = HashMap<String, Statement>;

pub fn evaluate(
    expr: &Expression,
    env: &Environment,
    func_env: &FunctionEnvironment,
) -> Result<Expression> {
    fn evaluate_list(
        args: &Vec<Box<Expression>>,
        env: &Environment,
        func_env: &FunctionEnvironment,
    ) -> Result<Vec<Expression>> {
        args.iter()
            .map(|arg| evaluate(arg, env, func_env))
            .collect()
    }

    fn exec_fun(
        func_name: &str,
        args: &Vec<Expression>,
        func_env: &FunctionEnvironment,
    ) -> Result<Expression> {
        fn build_environment_from_args(
            params: &Vec<String>,
            args: &Vec<Expression>,
        ) -> Result<Environment> {
            if params.len() != args.len() {
                anyhow::bail!(
                    "The number of arguments is not matched. params: {:?}, args: {:?}",
                    params,
                    args
                );
            }
            let mut env = HashMap::new();
            for (param, arg) in params.into_iter().zip(args.into_iter()) {
                env.insert(param.to_string(), arg.clone());
            }
            Ok(env)
        }

        let stmt = func_env
            .get(func_name)
            .ok_or_else(|| anyhow::anyhow!("Unknown function: {}", func_name))?;
        let func = match stmt {
            Statement::FuncDef { params, body } => (params, body),
            _ => anyhow::bail!("Expected to Statement::FuncDef {:?}", stmt),
        };
        let mut env = build_environment_from_args(func.0, args)?;
        env.insert(String::from("return"), Expression::Int { value: 0 });
        let env = execute(&*func.1, &env, func_env)?;
        env.get("return")
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Expected to return value"))
    }

    match expr {
        Expression::Var { name } => Ok(env[name].clone()),
        Expression::BinExp { op, lhs, rhs } => {
            let left = evaluate(&*lhs, env, func_env)?;
            let right = evaluate(&*rhs, env, func_env)?;
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
        Expression::Int { value } => Ok(Expression::Int { value: *value }),
        Expression::Call { name, args } => {
            exec_fun(&name, &evaluate_list(args, env, func_env)?, func_env)
        }
    }
}

pub fn execute(
    stmt: &Statement,
    env: &Environment,
    func_env: &FunctionEnvironment,
) -> Result<Environment> {
    match stmt {
        Statement::If { cond, then, els } => {
            let cond = evaluate(&*cond, env, func_env)?;
            let cond_value = if let Expression::Int { value } = cond {
                value
            } else {
                anyhow::bail!("Expected to Expression::Int {:?}", cond);
            };
            if cond_value != 0 {
                execute(&*then, env, func_env)
            } else {
                execute(&*els, env, func_env)
            }
        }
        Statement::While { cond, stmt } => {
            let mut current_env = env.clone();
            while let Expression::Int { value } = evaluate(&*cond, &current_env, func_env)? {
                if value == 0 {
                    break;
                }
                current_env = execute(&*stmt, &current_env, func_env)?;
            }
            Ok(current_env)
        }
        Statement::Assign { name, expr } => {
            let value = evaluate(&*expr, &env, func_env)?;
            let mut current_env = env.clone();
            current_env.insert(name.to_string(), value);
            Ok(current_env)
        }
        Statement::Sequence { stmts } => {
            let mut current_env = env.clone();
            for stmt in stmts {
                current_env = execute(&*stmt, &current_env, func_env)?;
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
