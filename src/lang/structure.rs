use std::mem;

use serde::Deserialize;

use crate::grammar;

#[derive(Deserialize, Debug)]
pub struct SandConfig {
    pub name: String,
    pub version: String,
}

#[derive(Debug)]
pub struct SandTree {
    files: Vec<grammar::File>,
    pub variables: Vec<grammar::var::Variable>,
    pub functions: Vec<grammar::function::Function>,
    pub function_calls: Vec<grammar::function_call::FunctionCall>,
}

impl SandTree {
    pub fn new() -> Self {
        SandTree {
            files: Vec::new(),
            variables: Vec::new(),
            functions: Vec::new(),
            function_calls: Vec::new(),
        }
    }

    pub fn add_file(&mut self, file: grammar::File) {
        self.files.push(file);
    }

    pub fn sort(mut self) -> Result<SandTree, String> {
        let files = mem::take(&mut self.files);
        println!("Files len {}", files.len());
        for file in files {
            self.variables.extend(file.variables);
            self.functions.extend(file.functions);
            self.function_calls.extend(file.function_calls);
        }
        Ok(self)
    }

    pub fn validate_function_calls(&self) -> Result<bool, String> {
        println!(
            "Validating {} function calls against {} functions",
            self.function_calls.len(),
            self.functions.len()
        );

        for call in &self.function_calls {
            println!("Validating call to function: {}", call.function_name);
            let function = self
                .functions
                .iter()
                .find(|f| f.name == call.function_name)
                .ok_or_else(|| format!("Function '{}' not found", call.function_name))?;

            call.validate_types(function)?;
        }
        Ok(true)
    }
}
