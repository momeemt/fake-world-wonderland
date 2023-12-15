use anyhow::Result;

use crate::ast::Expression;

pub fn eval_const(expr: Expression) -> Result<Expression> {
    match expr {
        Expression::BinExp { op, lhs, rhs } => {
            let left = eval_const(*lhs)?;
            let right = eval_const(*rhs)?;
            let left_value = if let Expression::Int { value } = left {
                value
            } else {
                anyhow::bail!("Expected to Expression::Int but {:?}", left);
            };
            let right_value = if let Expression::Int { value } = left {
                value
            } else {
                anyhow::bail!("Expected to Expression::Int but {:?}", right);
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
                _ => anyhow::bail!("Unknown op: {}", op),
            }
        }
        Expression::Int { value } => Ok(Expression::Int { value }),
        _ => anyhow::bail!("Unknown expression: {:?}", expr),
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::ast::Expression;

    use super::eval_const;

    #[test]
    fn four_arithmetic_ops1() -> Result<()> {
        let expr = Expression::BinExp {
            op: "/".to_string(),
            lhs: Box::new(Expression::BinExp {
                op: "*".to_string(),
                lhs: Box::new(Expression::Int { value: 2 }),
                rhs: Box::new(Expression::BinExp {
                    op: "-".to_string(),
                    lhs: Box::new(Expression::Int { value: 5 }),
                    rhs: Box::new(Expression::Int { value: 2 }),
                }),
            }),
            rhs: Box::new(Expression::Int { value: 4 }),
        };
        let res = eval_const(expr)?;
        if let Expression::Int { value } = res {
            assert_eq!(value, 1);
        } else {
            assert!(false, "Expected Expression::Int, got {:?}", res);
        }
        Ok(())
    }
}
