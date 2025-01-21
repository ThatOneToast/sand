use crate::{grammar::var::{Type, TypeNotation}, lang::Rule};

use super::function::Function;

#[derive(Debug, Clone)]
pub struct FunctionCall {
    pub function_name: String,
    pub parameters: Vec<Type>,
}

impl FunctionCall {
    pub fn from_pest(pair: pest::iterators::Pair<Rule>) -> Self {
        let mut inner = pair.into_inner();
        let function_name = inner.next().unwrap().as_str().to_string();

        let mut parameters = Vec::new();

        if let Some(params_pair) = inner.next() {
            for param in params_pair.into_inner() {
                // Get the value from the passing_param
                let value_pair = param.into_inner().next().unwrap();
                parameters.push(Type::from_pest(value_pair));
            }
        }

        FunctionCall {
            function_name,
            parameters,
        }
    }

    pub fn validate_types(&self, function: &Function) -> Result<(), String> {
        println!("\nValidating function call:");
        println!("Function name: {}", self.function_name);
        println!("Parameters provided: {:?}", self.parameters);
        println!("Expected parameters: {:?}", function.parameters);

        if self.parameters.len() != function.parameters.len() {
            return Err(format!(
                "Function {} expects {} parameters, but {} were passed",
                function.name,
                function.parameters.len(),
                self.parameters.len()
            ));
        }

        for (idx, (param_value, expected_param)) in
            self.parameters.iter().zip(&function.parameters).enumerate()
        {
            println!("\nChecking parameter {}:", idx);
            println!("Value: {:?}", param_value);
            println!("Expected type: {:?}", expected_param.type_notation);

            if !self.type_matches(param_value, &expected_param.type_notation) {
                return Err(format!(
                    "Parameter {} expects type {:?}, but got {:?}",
                    expected_param.name, expected_param.type_notation, param_value
                ));
            }
        }

        Ok(())
    }

    fn type_matches(&self, value: &Type, expected: &TypeNotation) -> bool {
        let result = match (value, expected) {
            (Type::String(_), TypeNotation::String) => true,
            (Type::Number(_), TypeNotation::Number) => true,
            (Type::Boolean(_), TypeNotation::Boolean) => true,
            _ => false,
        };
        println!("Type match result: {}", result);
        result
    }
}