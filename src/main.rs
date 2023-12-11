use anyhow::{Context, Result};

#[derive(Debug, Clone)]
enum StackOperation {
    Push,
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone)]
enum StackInstruction {
    Operation(StackOperation),
    Data(i32)
}

fn execute (instructions: Vec<StackInstruction>, stack_values: Vec<i32>) -> Result<i32> {
    let mut instructions = instructions.into_iter().rev().collect::<Vec<_>>();
    let mut stack = stack_values.to_vec();
    while let Some(instruction) = instructions.pop() {
        match instruction {
            StackInstruction::Operation(StackOperation::Push) => {
                let operand = match instructions.pop().context("stack is empty")? {
                    StackInstruction::Data(value) => value,
                    _ => anyhow::bail!("expected a data value"),
                };
                stack.push(operand);
            },
            StackInstruction::Operation(StackOperation::Add) => {
                let left = stack.pop().context("stack is empty")?;
                let right = stack.pop().context("stack is empty")?;
                stack.push(right + left);
            },
            StackInstruction::Operation(StackOperation::Sub) => {
                let left = stack.pop().context("stack is empty")?;
                let right = stack.pop().context("stack is empty")?;
                stack.push(right - left);
            },
            StackInstruction::Operation(StackOperation::Mul) => {
                let left = stack.pop().context("stack is empty")?;
                let right = stack.pop().context("stack is empty")?;
                stack.push(right * left);
            },
            StackInstruction::Operation(StackOperation::Div) => {
                let left = stack.pop().context("stack is empty")?;
                let right = stack.pop().context("stack is empty")?;
                stack.push(right / left);
            },
            StackInstruction::Data(_) => {
                anyhow::bail!("expected a operation value")
            }
        };
    }
    stack.last().copied().context("stack is empty")
}

fn main() -> Result<()> {
    let res = execute(vec![
        StackInstruction::Operation(StackOperation::Push),
        StackInstruction::Data(2),
        StackInstruction::Operation(StackOperation::Add),
    ], vec![1])?;
    println!("{}", res);

    let res2 = execute(vec![
        StackInstruction::Operation(StackOperation::Push),
        StackInstruction::Data(5),
        StackInstruction::Operation(StackOperation::Push),
        StackInstruction::Data(2),
        StackInstruction::Operation(StackOperation::Sub),
        StackInstruction::Operation(StackOperation::Mul),
        StackInstruction::Operation(StackOperation::Push),
        StackInstruction::Data(4),
        StackInstruction::Operation(StackOperation::Div),
    ], vec![2])?;
    println!("{}", res2);
    
    Ok(())
}

