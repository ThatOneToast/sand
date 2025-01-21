use crate::{
    grammar::{
        var::{Scope, Variable},
        FunctionCall,
    },
    lang::Rule,
};

use super::{collection::CollectionId, if_stmt::IfStatement, var::TypeNotation, Statement};

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub variables: Vec<Variable>,
    pub function_calls: Vec<FunctionCall>,
    pub if_statements: Vec<IfStatement>,
    pub collections: Vec<CollectionId>,
}

impl Function {
    pub fn from_pest(pair: pest::iterators::Pair<Rule>) -> Self {
        let mut inner = pair.into_inner();
        let name = inner.next().unwrap().as_str().to_string();

        let mut parameters = Vec::new();
        let mut variables = Vec::new();
        let mut function_calls = Vec::new();
        let mut if_statements = Vec::new();
        let mut collections = Vec::new();

        for item in inner {
            match item.as_rule() {
                Rule::params => {
                    parameters = item.into_inner().map(Parameter::from_pest).collect();
                }
                Rule::func_block => {
                    for statement in item.into_inner() {
                        match statement.as_rule() {
                            Rule::variable => {
                                variables.push(Variable::from_pest_scoped(
                                    statement,
                                    Scope::Function(name.to_string()),
                                ));
                            }
                            Rule::function_call => {
                                function_calls.push(FunctionCall::from_pest(statement));
                            }
                            Rule::if_statement => {
                                if_statements.push(IfStatement::from_pest(statement));
                            }
                            Rule::collection => {
                                collections.push(CollectionId::from_pest(statement));
                            }
                            _ => println!(
                                "Ignored rule in function block: {:?}",
                                statement.as_rule()
                            ),
                        }
                    }
                }
                _ => println!("Ignored rule in function: {:?}", item.as_rule()),
            }
        }

        Function {
            name,
            parameters,
            variables,
            function_calls,
            if_statements,
            collections,
        }
    }

    pub fn validate_statements(&self, file: &super::File) -> Result<(), String> {
        // Validate function calls
        for call in &self.function_calls {
            let target_function = file
                .functions
                .iter()
                .find(|f| f.name == call.function_name)
                .ok_or_else(|| format!("Function '{}' not found", call.function_name))?;
            call.validate_types(target_function)?;
        }

        // Validate if statements
        for if_stmt in &self.if_statements {
            for stmt in &if_stmt.block {
                match stmt {
                    Statement::FunctionCall(call) => {
                        let target_function = file
                            .functions
                            .iter()
                            .find(|f| f.name == call.function_name)
                            .ok_or_else(|| {
                                format!("Function '{}' not found", call.function_name)
                            })?;
                        call.validate_types(target_function)?;
                    }
                    Statement::Variable(_) => {
                        // Add variable validation if needed
                    }
                    Statement::Collection(col) => {
                        col.validate()?;
                    }
                }
            }
        }

        // Validate collections
        for collection in &self.collections {
            collection.validate()?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub type_notation: TypeNotation,
}

impl Parameter {
    pub fn from_pest(pair: pest::iterators::Pair<Rule>) -> Self {
        let mut inner = pair.into_inner();
        let name = inner.next().unwrap().as_str().to_string();
        let type_notation = TypeNotation::from_pest(inner.next().unwrap());

        Parameter {
            name,
            type_notation,
        }
    }
}

#[derive(Debug, Clone)]
pub enum FunctionStatement {
    Standard(Statement),
    If(IfStatement),
}
