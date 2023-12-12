use anyhow::Result;

use crate::ast::Expression;

pub fn apply_rule(expr: &Expression) -> Result<Expression> {
    match expr {
        Expression::BinExp { op, lhs, rhs } => {
            match (&**lhs, &**rhs) {
                (Expression::Int { value: left_val }, Expression::Int { value: right_val }) => {
                    if op == "+" {
                        Ok(Expression::Int{ value: left_val + right_val })
                    } else if op == "-" {
                        Ok(Expression::Int{ value: left_val - right_val })
                    } else if op == "*" {
                        Ok(Expression::Int{ value: left_val * right_val })
                    } else if op == "/" {
                        Ok(Expression::Int{ value: left_val / right_val })
                    } else {
                        anyhow::bail!("Unknown op: {}", op)
                    }
                },
                (Expression::BinExp { .. }, _) => {
                    let processed_lhs = apply_rule(lhs)?;
                    Ok(Expression::BinExp { op: op.to_string(), lhs: Box::new(processed_lhs), rhs: rhs.clone() })
                },
                (_, Expression::BinExp { .. }) => {
                    let processed_rhs = apply_rule(rhs)?;
                    Ok(Expression::BinExp { op: op.to_string(), lhs: lhs.clone(), rhs: Box::new(processed_rhs) })
                },
                (_, _) => anyhow::bail!("No applicable rule for: {:?}", expr)
            }
        },
        _ => anyhow::bail!("No applicable rule for: {:?}", expr)
    }
}

pub fn rewrite_loop(expr: Expression) -> Result<Expression> { 
    match expr {
        Expression::Int { value } => Ok(Expression::Int { value }),
        non_int @ _ => rewrite_loop(apply_rule(&non_int)?),
    } 
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::ast::Expression;

    use super::rewrite_loop;

    #[test]
    fn four_arithmetic_ops1() -> Result<()> {
        let expr = Expression::BinExp { op: "/".to_string(),
            lhs: Box::new(Expression::BinExp { op: "*".to_string(),
                lhs: Box::new(Expression::Int { value: 2 }),
                rhs: Box::new(Expression::BinExp { op: "-".to_string(),
                    lhs: Box::new(Expression::Int { value: 5 }),
                    rhs: Box::new(Expression::Int { value: 2 }),
                })
            }),
            rhs: Box::new(Expression::Int { value: 4 })
        };
        let res = rewrite_loop(expr)?;
        if let Expression::Int { value } = res {
            assert_eq!(value, 1);
        } else {
            assert!(false, "Expected Expression::Int, got {:?}", res);
        }
        Ok(())
    }
}
