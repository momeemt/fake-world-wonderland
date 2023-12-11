use anyhow::{Context, Result};

#[derive(Debug, Clone)]
enum StackOperation {
    Push,
    Add,
}

#[derive(Debug, Clone)]
enum StackInstruction {
    Operation(StackOperation),
    Data(i32)
}

fn execute (instructions: Vec<StackInstruction>, stack_values: Vec<i32>) -> Result<i32> {
    let mut instructions = instructions.to_vec();
    instructions.reverse();
    let mut stack = stack_values.to_vec();
    while instructions.len() > 0 {
        let code = instructions.pop().context("stack is empty")?;
        match code {
            StackInstruction::Operation(op) => {
                match op {
                    StackOperation::Push => {
                        let operand = match instructions.pop().context("stack is empty")? {
                            StackInstruction::Data(value) => value,
                            _ => anyhow::bail!("expected a data value"),
                        };
                        stack.push(operand);
                    },
                    StackOperation::Add => {
                        let left = stack.pop().context("stack is empty")?;
                        let right = stack.pop().context("stack is empty")?;
                        stack.push(right + left);
                    }
                }
            },
            StackInstruction::Data(_) => {
                anyhow::bail!("expected a operation value")
            }
        };
    }
    let last = stack.last().context("stack is empty")?;
    Ok(*last)
}

fn main() -> Result<()> {
    let res = execute(vec![
        StackInstruction::Operation(StackOperation::Push),
        StackInstruction::Data(2),
        StackInstruction::Operation(StackOperation::Add),
    ], vec![1])?;
    println!("{}", res);
    Ok(())
}

