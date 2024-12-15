#[cfg(test)]
mod tests {
    use crate::{datapack::ExecuteSubcommand, lang::Statement};

    #[test]
    fn testing_execute() {
        let statement = Statement::Execute(vec![
            ExecuteSubcommand::As("@p".to_string()),
            ExecuteSubcommand::At("@s".to_string()),
            ExecuteSubcommand::Run(Box::new(Statement::Say("Hello".to_string()))),
        ]);

        assert_eq!(statement.to_string(), "execute as @p at @s run say Hello");
    }

    #[test]
    fn testing_complex_execute() {
        let statement = Statement::Execute(vec![
            ExecuteSubcommand::As("@p".to_string()),
            ExecuteSubcommand::At("@s".to_string()),
            ExecuteSubcommand::If(crate::datapack::Condition::Entity(
                "@e[type=zombie]".to_string(),
            )),
            ExecuteSubcommand::Run(Box::new(Statement::Say("Found a zombie!".to_string()))),
        ]);

        assert_eq!(
            statement.to_string(),
            "execute as @p at @s if entity @e[type=zombie] run say Found a zombie!"
        );
    }

    #[test]
    fn testing_nested_execute() {
        let nested_execute = Statement::Execute(vec![
            ExecuteSubcommand::As("@s".to_string()),
            ExecuteSubcommand::Run(Box::new(Statement::Say("Nested".to_string()))),
        ]);

        let statement = Statement::Execute(vec![
            ExecuteSubcommand::As("@p".to_string()),
            ExecuteSubcommand::Run(Box::new(nested_execute)),
        ]);

        assert_eq!(
            statement.to_string(),
            "execute as @p run execute as @s run say Nested"
        );
    }
}
