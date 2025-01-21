use collection::CollectionId;
use function::Function;
use function_call::FunctionCall;
use var::Variable;

use crate::lang::Rule;

pub mod var;
pub mod if_stmt;
pub mod function;
pub mod function_call;
pub mod collection;
pub mod math;

pub mod tests;

#[derive(Debug, Clone)]
pub enum Statement {
    Variable(Variable),
    FunctionCall(FunctionCall),
    Collection(CollectionId)
}

impl Statement {
    pub fn from_pest(pair: pest::iterators::Pair<Rule>) -> Self {
        let inner = pair.into_inner().next().unwrap();
        match inner.as_rule() {
            Rule::variable => Statement::Variable(Variable::from_pest(inner)),
            Rule::function_call => Statement::FunctionCall(FunctionCall::from_pest(inner)),
            Rule::collection => Statement::Collection(CollectionId::from_pest(inner)),
            _ => unreachable!("Unexpected statement rule: {:?}", inner.as_rule()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct File {
    pub variables: Vec<Variable>,
    pub functions: Vec<Function>,
    pub function_calls: Vec<FunctionCall>,
}

impl File {
    pub fn from_pest(pair: pest::iterators::Pair<Rule>) -> Self {
        let mut variables = Vec::new();
        let mut functions = Vec::new();
        let mut function_calls = Vec::new();

        println!("Parsing file contents:"); // Add debug print
        for item in pair.into_inner() {
            println!("Found item with rule: {:?}", item.as_rule()); // Add debug print
            match item.as_rule() {
                Rule::variable => {
                    println!("Found variable: {}", item.as_str()); // Add debug print
                    variables.push(Variable::from_pest(item));
                }
                Rule::function => {
                    println!("Found function: {}", item.as_str()); // Add debug print
                    functions.push(Function::from_pest(item));
                }
                Rule::function_call => {
                    println!("Found function call: {}", item.as_str()); // Add debug print
                    function_calls.push(FunctionCall::from_pest(item));
                }
                _ => println!("------------>>>>>>>>> Ignored rule: {:?}", item.as_rule()),
            }
        }

        println!(
            "Parsed {} variables \n{} functions \n{} function_calls",
            variables.len(),
            functions.len(),
            function_calls.len()
        );

        File {
            variables,
            functions,
            function_calls,
        }
    }

    pub fn validate_function_calls(&self) -> Result<(), String> {
        for call in &self.function_calls {
            // Find the corresponding function definition
            let function = self
                .functions
                .iter()
                .find(|f| f.name == call.function_name)
                .ok_or_else(|| format!("Function '{}' not found", call.function_name))?;

            // Validate the parameter types
            call.validate_types(function)?;
        }
        Ok(())
    }

}
