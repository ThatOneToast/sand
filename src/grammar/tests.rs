#[cfg(test)]
mod tests {
    use if_stmt::{ComparisonOperator, Condition, ConditionValue};
    use pest::Parser;
    use var::Type;

    use crate::{
        grammar::*,
        lang::{parse, Rule, SandParser},
    };

    #[test]
    fn test_individual_rules() {
        // Test variable parsing
        let variable_input = "let x = 42";
        let result = SandParser::parse(Rule::file, variable_input);
        assert!(
            result.is_ok(),
            "Failed to parse variable: {:?}",
            result.err()
        );

        // Test function parsing
        let function_input = "fn test(x: Number) { let y = 42 }";
        let result = SandParser::parse(Rule::file, function_input);
        assert!(
            result.is_ok(),
            "Failed to parse function: {:?}",
            result.err()
        );

        // Test complete file with various whitespace
        let complete_input = r#"
                let name = 'Toast'
                let age = 35

                fn test(x: Number) {
                    let y = 42
                }
            "#
        .trim();
        let result = SandParser::parse(Rule::file, complete_input);
        assert!(
            result.is_ok(),
            "Failed to parse complete file: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parse_tree() {
        let input = include_str!("/Users/austinaleshire/Projects/sand/main.sand");

        let tree = parse(input).unwrap().sort().unwrap();

        assert!(!tree.variables.is_empty(), "No variables were parsed");
        assert!(!tree.functions.is_empty(), "No functions were parsed");
    }

    #[test]
    fn test_function_call_types() {
        // Test valid function call
        let input = r#"
            fn test(name: String, count: Number) {
                let x = 5
            }
            #test('hello', 42)
        "#;

        let mut result = parse(input).unwrap();
        result = result.sort().unwrap(); // Make sure to sort before validating

        println!("Functions in tree: {:?}", result.functions);
        println!("Function calls in tree: {:?}", result.function_calls);

        let validation_result = result.validate_function_calls();
        assert!(
            validation_result.is_ok(),
            "Valid function call failed validation: {:?}",
            validation_result
        );

        // Test invalid function call
        let input_with_error = r#"
            fn test(name: String, count: Number) {
                let x = 5
            }
            #test(42, 'wrong')
        "#;

        let mut result = parse(input_with_error).unwrap();
        result = result.sort().unwrap(); // Make sure to sort before validating

        println!("Functions in tree: {:?}", result.functions);
        println!("Function calls in tree: {:?}", result.function_calls);

        let validation_result = result.validate_function_calls();
        assert!(
            validation_result.is_err(),
            "Invalid function call passed validation when it should have failed. Parameters: {:?}",
            result
                .function_calls
                .first()
                .expect("No function calls found")
        );
    }

    #[test]
    fn test_if_statement_in_function() {
        let input = r#"
            fn test(x: Number) {
                let before = 'before'
                if x > 5 {
                    let y = 42
                    #someFunc('hello')
                }
                let after = 'after'
            }

            fn someFunc(msg: String) {
                let x = 1
            }
        "#
        .trim();

        let mut result = parse(input).unwrap();
        result = result.sort().unwrap();

        assert!(!result.functions.is_empty(), "No functions were parsed");
        assert!(
            result.validate_function_calls().unwrap(),
            "Function calls failed validation"
        );

        let test_func = result
            .functions
            .iter()
            .find(|f| f.name == "test")
            .expect("test function not found");

        // Check variables are in correct order
        assert_eq!(test_func.variables[0].identifier, "before");
        assert_eq!(test_func.variables[1].identifier, "after");

        // Check if statement
        let if_stmt = &test_func.if_statements[0];
        match &if_stmt.condition {
            Condition::Comparison {
                left,
                operator,
                right,
            } => {
                // Check identifier 'x'
                match left.as_ref() {
                    ConditionValue::Identifier(id) => assert_eq!(id, "x"),
                    _ => panic!("Expected identifier 'x'"),
                }
                // Check operator '>'
                assert!(matches!(operator, ComparisonOperator::GreaterThan));
                // Check number '5'
                match right.as_ref() {
                    ConditionValue::Literal(Type::Number(num)) => assert_eq!(num, &5.0),
                    _ => panic!("Expected number 5"),
                }
            }
            _ => panic!("Expected comparison condition"),
        }
    }
}
