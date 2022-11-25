//@author Stanislav Polaniev <spolanyev@gmail.com>

use crate::interfaces::validator_interface::ValidatorInterface;
use std::collections::HashMap;

pub struct RqlValidator {
    closing_parts: HashMap<char, char>,
    opening_parts: HashMap<char, char>,
    verbosity_manager: Option<fn(String)>,
}

impl ValidatorInterface for RqlValidator {
    fn is_valid(&self, data: String) -> bool {
        let operators = match self.get_operators(data) {
            Ok(operators) => operators,
            Err(error_message) => {
                self.process_error_message(error_message);
                return false;
            }
        };
        self.is_operators_valid(self.add_nested_nodes_quantity(operators))
    }
}

impl RqlValidator {
    pub fn new(verbosity_manager: Option<fn(String)>) -> Self {
        Self {
            closing_parts: HashMap::from([('(', ')')]),
            opening_parts: HashMap::from([(')', '(')]),
            verbosity_manager,
        }
    }

    fn process_error_message(&self, message: String) {
        if self.verbosity_manager.is_some() {
            self.verbosity_manager.expect("We checked it a line above")(message);
        }
    }

    fn get_operators(
        &self,
        rql_statement: String,
    ) -> Result<Vec<(String, Option<String>, usize)>, String> {
        let mut result: Vec<(String, Option<String>, usize)> = vec![];

        let mut stack = vec![];

        let mut is_inside_parentheses = false;
        let mut operator = "".to_owned();
        let mut operator_content = "".to_owned();
        let mut level: usize = 0;

        for char in rql_statement.chars() {
            //opening detected
            if let Some(closing_part) = self.closing_parts.get(&char) {
                stack.push(closing_part);
                level += 1;
                is_inside_parentheses = true;
                result.push((operator, None, level));
                operator = "".to_owned();
                operator_content = "".to_owned();
                continue;
            }

            if is_inside_parentheses {
                operator_content += &char.to_string();
            }

            if ',' != char {
                operator += &char.to_string();
            }

            //closing detected
            if self.opening_parts.get(&char).is_some() {
                if stack.pop() != Some(&char) {
                    return Err("Invalid closing parentheses count".to_owned());
                }
                level -= 1;
                operator = "".to_owned();
                operator_content.pop();

                if operator_content.is_empty() {
                    continue;
                }

                let last = result
                    .last_mut()
                    .expect("We have at least an operator before the value");
                *last = (last.0.clone(), Some(operator_content), last.2);
                operator_content = "".to_owned();
            }
        }

        if !stack.is_empty() {
            return Err("Invalid opening parentheses count".to_owned());
        }

        Ok(result)
    }

    fn add_nested_nodes_quantity(
        &self,
        operators: Vec<(String, Option<String>, usize)>,
    ) -> Vec<(String, Option<String>, usize, usize)> {
        operators
            .iter()
            .map(|x: &(String, Option<String>, usize)| {
                if x.1.is_none() {
                    let target_level = x.2 + 1;
                    let mut count_nested = 0;
                    for x in operators.iter() {
                        let (_, _, level) = x;
                        if target_level == *level {
                            count_nested += 1;
                        }
                    }
                    (x.0.clone(), x.1.clone(), x.2, count_nested)
                } else {
                    (x.0.clone(), x.1.clone(), x.2, 0usize)
                }
            })
            .collect()
    }

    fn is_operators_valid(&self, operators: Vec<(String, Option<String>, usize, usize)>) -> bool {
        let result = || {
            for x in operators.iter() {
                let (node, inner_value, level, nested_quantity) = x;
                if !(match node.as_str() {
                    //operator exists(property)
                    "exists" => {
                        if !{
                            0 == *nested_quantity
                                && inner_value.is_some()
                                && 0 == inner_value
                                    .as_ref()
                                    .expect("We checked few lines above")
                                    .matches(',')
                                    .count()
                        } {
                            self.process_error_message(
                                format!("Operator `{}` should not contain nested parentheses, must have a property, the property should not contain a comma", node),
                            );
                            return false;
                        }
                        true
                    }

                    //comparison operators e.g., eq(property,value)
                    "eq" | "ne" | "lt" | "gt" | "le" | "ge" => {
                        if !{
                            0 == *nested_quantity
                                && inner_value.is_some()
                                && 1 == inner_value
                                    .as_ref()
                                    .expect("We checked few lines above")
                                    .matches(',')
                                    .count()
                        } {
                            self.process_error_message(format!(
                                "Operator `{}` should not contain nested parentheses, must have a property and a value",
                                node
                            ));
                            return false;
                        }
                        true
                    }

                    //operator like(property,pattern)
                    "like" => {
                        if !{
                            0 == *nested_quantity
                                && inner_value.is_some()
                                && 1 == inner_value
                                    .as_ref()
                                    .expect("We checked few lines above")
                                    .matches(',')
                                    .count()
                        } {
                            self.process_error_message(format!(
                                "Operator `{}` should not contain nested parentheses, must have a property and a pattern",
                                node
                            ));
                            return false;
                        }
                        true
                    }

                    //list operators e.g., in(property,(value1,...))
                    "in" | "out" => {
                        if !{ 1 == *nested_quantity } {
                            self.process_error_message(format!(
                                "Operator `{}` should have 1 nested parentheses block",
                                node
                            ));
                            return false;
                        }
                        true
                    }

                    //logical operator not(query)
                    "not" => {
                        if !{ 1 == *nested_quantity } {
                            self.process_error_message(format!(
                                "Operator `{}` should have 1 nested query",
                                node
                            ));
                            return false;
                        }
                        true
                    }
                    //other logical operators e.g., and(query1,query2,...)
                    "and" | "or" => {
                        if !{ 1 < *nested_quantity } {
                            self.process_error_message(format!(
                                "Operator `{}` should have at least 2 nested queries",
                                node
                            ));
                            return false;
                        }
                        true
                    }
                    _ => inner_value.is_some() && *level > 1,
                }) {
                    self.process_error_message(format!(
                        "Block `{}` should have a value and be nested",
                        node
                    ));
                    return false;
                }
            }
            true
        };

        let result = result();
        /*
        if !result {
            println!("{:#?}", operators);
        }
        */
        result
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn check_is_broken() {
        let rql_validator: Box<dyn ValidatorInterface> = Box::new(RqlValidator::new(None));

        let invalid_rql_statement = "))".to_owned();
        assert!(!rql_validator.is_valid(invalid_rql_statement));
    }

    #[test]
    fn check_is_error_message_exist() {
        fn handle_error(error_message: String) {
            assert!(!error_message.is_empty());
        }

        let rql_validator: Box<dyn ValidatorInterface> =
            Box::new(RqlValidator::new(Some(handle_error)));

        let invalid_rql_statement = "))".to_owned();
        rql_validator.is_valid(invalid_rql_statement);
    }

    #[test]
    fn check_valid() {
        fn print_error(error_message: String) {
            eprintln!("Error: -------------------------------- {}", error_message);
        }

        let rql_validator: Box<dyn ValidatorInterface> =
            Box::new(RqlValidator::new(Some(print_error)));

        let rql_statement = "exists(product.status)".to_owned();
        assert!(rql_validator.is_valid(rql_statement));

        let rql_statement = "eq(product.status,new)".to_owned();
        assert!(rql_validator.is_valid(rql_statement));

        let rql_statement = "lt(product.createdAt,2022-11-25)".to_owned();
        assert!(rql_validator.is_valid(rql_statement));

        let rql_statement = "like(product.name,*str*)".to_owned();
        assert!(rql_validator.is_valid(rql_statement));

        let rql_statement = "in(status,(new,processing))".to_owned();
        assert!(rql_validator.is_valid(rql_statement));

        let rql_statement = "in(status,(new))".to_owned();
        assert!(rql_validator.is_valid(rql_statement));

        let rql_statement = "and(eq(id,12),like(name,*str*))".to_owned();
        assert!(rql_validator.is_valid(rql_statement));

        let rql_statement = "and(eq(id,12),like(name,*str*),eq(name, John))".to_owned();
        assert!(rql_validator.is_valid(rql_statement));

        let rql_statement = "not(eq(product.name,astra))".to_owned();
        assert!(rql_validator.is_valid(rql_statement));

        let rql_statement = "status,new".to_owned();
        assert!(rql_validator.is_valid(rql_statement));
    }

    #[test]
    fn check_invalid() {
        fn print_error(error_message: String) {
            eprintln!("Error: -------------------------------- {}", error_message);
        }

        let rql_validator: Box<dyn ValidatorInterface> =
            Box::new(RqlValidator::new(Some(print_error)));

        let rql_statement = "exists(eq(product.status,new))".to_owned();
        assert!(!rql_validator.is_valid(rql_statement));

        let rql_statement = "eq(product.status)".to_owned();
        assert!(!rql_validator.is_valid(rql_statement));

        let rql_statement = "lt(product.createdAt,2022-11-25,20:38)".to_owned();
        assert!(!rql_validator.is_valid(rql_statement));

        let rql_statement = "like(product.name)".to_owned();
        assert!(!rql_validator.is_valid(rql_statement));

        let rql_statement = "in(status)".to_owned();
        assert!(!rql_validator.is_valid(rql_statement));

        let rql_statement = "in(status,(active),(new))".to_owned();
        assert!(!rql_validator.is_valid(rql_statement));

        let rql_statement = "and(eq(id,12))".to_owned();
        assert!(!rql_validator.is_valid(rql_statement));

        let rql_statement = "and(name, John)".to_owned();
        assert!(!rql_validator.is_valid(rql_statement));

        let rql_statement = "not(product.name,astra)".to_owned();
        assert!(!rql_validator.is_valid(rql_statement));

        let rql_statement = "unknown.operator(value)".to_owned();
        assert!(!rql_validator.is_valid(rql_statement));
    }

    #[test]
    fn get_operators() {
        let rql_validator = RqlValidator::new(None);

        let rql_statement = "eq(name,John)".to_owned();
        let expected = vec![("eq".to_owned(), Some("name,John".to_owned()), 1)];
        assert_eq!(
            expected,
            rql_validator.get_operators(rql_statement).unwrap()
        );

        let rql_statement = "or(and(eq(name,John),eq(surname,Smith)),eq(surname,Doe))".to_owned();
        let expected = vec![
            ("or".to_owned(), None, 1),
            ("and".to_owned(), None, 2),
            ("eq".to_owned(), Some("name,John".to_owned()), 3),
            ("eq".to_owned(), Some("surname,Smith".to_owned()), 3),
            ("eq".to_owned(), Some("surname,Doe".to_owned()), 2),
        ];
        assert_eq!(
            expected,
            rql_validator.get_operators(rql_statement).unwrap()
        );

        let rql_statement = "not(in(name,(John,Jackson,Liam)))".to_owned();
        let expected = vec![
            ("not".to_owned(), None, 1),
            ("in".to_owned(), None, 2),
            ("name".to_owned(), Some("John,Jackson,Liam".to_owned()), 3),
        ];
        assert_eq!(
            expected,
            rql_validator.get_operators(rql_statement).unwrap()
        );
    }

    #[test]
    fn add_nested_quantity() {
        let rql_validator = RqlValidator::new(None);

        let rql_statement = "eq(name,John)".to_owned();
        let expected = vec![("eq".to_owned(), Some("name,John".to_owned()), 1, 0)];
        assert_eq!(
            expected,
            rql_validator
                .add_nested_nodes_quantity(rql_validator.get_operators(rql_statement).unwrap())
        );

        let rql_statement = "or(and(eq(name,John),eq(surname,Smith)),eq(surname,Doe))".to_owned();
        let expected = vec![
            ("or".to_owned(), None, 1, 2),
            ("and".to_owned(), None, 2, 2),
            ("eq".to_owned(), Some("name,John".to_owned()), 3, 0),
            ("eq".to_owned(), Some("surname,Smith".to_owned()), 3, 0),
            ("eq".to_owned(), Some("surname,Doe".to_owned()), 2, 0),
        ];
        assert_eq!(
            expected,
            rql_validator
                .add_nested_nodes_quantity(rql_validator.get_operators(rql_statement).unwrap())
        );

        let rql_statement = "not(in(name,(John,Jackson,Liam)))".to_owned();
        let expected = vec![
            ("not".to_owned(), None, 1, 1),
            ("in".to_owned(), None, 2, 1),
            (
                "name".to_owned(),
                Some("John,Jackson,Liam".to_owned()),
                3,
                0,
            ),
        ];
        assert_eq!(
            expected,
            rql_validator
                .add_nested_nodes_quantity(rql_validator.get_operators(rql_statement).unwrap())
        );
    }
}
