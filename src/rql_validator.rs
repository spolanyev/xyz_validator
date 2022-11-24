//@author Stanislav Polaniev <spolanyev@gmail.com>

use crate::interfaces::validator_interface::ValidatorInterface;
use std::collections::HashMap;

pub struct RqlValidator {
    closing_parts: HashMap<char, char>,
    opening_parts: HashMap<char, char>,
    verbosity_manager: Option<fn(&str)>,
}

impl ValidatorInterface for RqlValidator {
    fn is_valid(&self, data: String) -> bool {
        if !self.is_parentheses_matched(data.as_str()) {
            return false;
        }
        self.is_operators_valid(self.add_nested_nodes_quantity(self.get_operators(data)))
    }
}

impl RqlValidator {
    pub fn new(verbosity_manager: Option<fn(&str)>) -> Self {
        Self {
            closing_parts: HashMap::from([('(', ')')]),
            opening_parts: HashMap::from([(')', '(')]),
            verbosity_manager,
        }
    }

    fn process_error_message(&self, message: &str) {
        if self.verbosity_manager.is_some() {
            self.verbosity_manager.expect("We checked it already")(message);
        }
    }

    fn is_parentheses_matched(&self, rql_statement: &str) -> bool {
        let mut stack = vec![];

        for char in rql_statement.chars() {
            //opening detected
            if let Some(closing_part) = self.closing_parts.get(&char) {
                stack.push(closing_part);
                continue;
            }

            //closing detected
            if self.opening_parts.get(&char).is_some() {
                if stack.pop() != Some(&char) {
                    self.process_error_message("Invalid closing parentheses count");
                    return false;
                }
            }
        }

        if stack.is_empty() {
            return true;
        }
        self.process_error_message("Invalid opening parentheses count");
        false
    }

    fn get_operators(&self, rql_statement: String) -> Vec<(String, Option<String>, usize)> {
        let mut result: Vec<(String, Option<String>, usize)> = vec![];

        let mut is_inside_parentheses = false;
        let mut operator = "".to_owned();
        let mut operator_content = "".to_owned();
        let mut level: usize = 0;

        for char in rql_statement.chars() {
            //opening detected
            if self.closing_parts.get(&char).is_some() {
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
                level -= 1;
                operator = "".to_owned();
                operator_content.pop();

                if operator_content.is_empty() {
                    continue;
                }
                let last = result
                    .last_mut()
                    .expect("We have at least operator before the value");
                *last = (last.0.clone(), Some(operator_content), last.2);
                operator_content = "".to_owned();
            }
        }
        result
    }

    fn is_operators_valid(&self, operators: Vec<(String, Option<String>, usize, usize)>) -> bool {
        for x in operators.iter() {
            let (node, inner_value, level, nested_quantity) = x;
            if !(match node.as_str() {
                //eq(field1,value1)
                "eq" | "ge" | "gt" | "le" | "lt" | "ne" => {
                    if !{ 0 == *nested_quantity } {
                        self.process_error_message(
                            format!("Node '{}' should not have nested parentheses", node).as_str(),
                        );
                        return false;
                    }
                    true
                }

                //eqf(field1)
                "eqf" | "eqt" | "eqn" | "ie" => {
                    if !{
                        0 == *nested_quantity
                            && inner_value.is_some()
                            && 0 == inner_value
                                .as_ref()
                                .expect("We checked before")
                                .matches(',')
                                .count()
                    } {
                        self.process_error_message(
                            format!("Node '{}' should not have nested parentheses, must contain a field, the field should not contain a comma", node).as_str(),
                        );
                        return false;
                    }
                    true
                }

                //in(field1,(value1,value2))
                "in" | "out" => {
                    if !{ 1 == *nested_quantity } {
                        self.process_error_message(
                            format!("Node '{}' should have 1 nested parentheses block", node)
                                .as_str(),
                        );
                        return false;
                    }
                    true
                }
                //not(node1)
                "not" => {
                    if !{ 1 == *nested_quantity } {
                        self.process_error_message(
                            format!("Node '{}' should have 1 nested node", node).as_str(),
                        );
                        return false;
                    }
                    true
                }
                //and(node1,node2)
                "and" | "or" => {
                    if !{ 2 == *nested_quantity } {
                        self.process_error_message(
                            format!("Node '{}' should have 2 nested nodes", node).as_str(),
                        );
                        return false;
                    }
                    true
                }
                _ => inner_value.is_some() && level > &1,
            }) {
                self.process_error_message(
                    format!("Block '{}' should have a value and be nested", node).as_str(),
                );
                //println!("{:#?}", operators);
                return false;
            }
        }
        //println!("{:#?}", operators);
        true
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
                        if &target_level == level {
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
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn check_parentheses() {
        let rql_validator = RqlValidator::new(None);

        let rql_statement = "eq(name,John)".to_owned();
        assert!(rql_validator.is_valid(rql_statement));

        let rql_statement = "and(eq(name,John),eq(surname,Smith))".to_owned();
        assert!(rql_validator.is_valid(rql_statement));

        let rql_statement = "or(and(eq(name,John),eq(surname,Smith)),eq(surname,Doe))".to_owned();
        assert!(rql_validator.is_valid(rql_statement));
    }

    #[test]
    fn get_operators() {
        let rql_validator = RqlValidator::new(None);

        let rql_statement = "eq(name,John)".to_owned();
        let expected = vec![("eq".to_owned(), Some("name,John".to_owned()), 1)];
        assert_eq!(expected, rql_validator.get_operators(rql_statement));

        let rql_statement = "or(and(eq(name,John),eq(surname,Smith)),eq(surname,Doe))".to_owned();
        let expected = vec![
            ("or".to_owned(), None, 1),
            ("and".to_owned(), None, 2),
            ("eq".to_owned(), Some("name,John".to_owned()), 3),
            ("eq".to_owned(), Some("surname,Smith".to_owned()), 3),
            ("eq".to_owned(), Some("surname,Doe".to_owned()), 2),
        ];
        assert_eq!(expected, rql_validator.get_operators(rql_statement));

        let rql_statement = "not(in(name,(John,Jackson,Liam)))".to_owned();
        let expected = vec![
            ("not".to_owned(), None, 1),
            ("in".to_owned(), None, 2),
            ("name".to_owned(), Some("John,Jackson,Liam".to_owned()), 3),
        ];
        assert_eq!(expected, rql_validator.get_operators(rql_statement));
    }

    #[test]
    fn add_nested_quantity() {
        let rql_validator = RqlValidator::new(None);

        let rql_statement = "eq(name,John)".to_owned();
        let expected = vec![("eq".to_owned(), Some("name,John".to_owned()), 1, 0)];
        assert_eq!(
            expected,
            rql_validator.add_nested_nodes_quantity(rql_validator.get_operators(rql_statement))
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
            rql_validator.add_nested_nodes_quantity(rql_validator.get_operators(rql_statement))
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
            rql_validator.add_nested_nodes_quantity(rql_validator.get_operators(rql_statement))
        );
    }

    #[test]
    fn check_is_operators_valid() {
        let rql_validator = RqlValidator::new(None);
        let rql_statement = "not(in(name,(John,Jackson,Liam)))".to_owned();
        let result =
            rql_validator.add_nested_nodes_quantity(rql_validator.get_operators(rql_statement));

        assert!(rql_validator.is_operators_valid(result));
    }

    #[test]
    fn test_valid() {
        let rql_validator = RqlValidator::new(None);

        let rql_statement = "not(in(name,(John,Jackson,Liam)))".to_owned();
        assert!(rql_validator.is_valid(rql_statement));

        let rql_statement = "in(name,(John,Jackson,Liam))".to_owned();
        assert!(rql_validator.is_valid(rql_statement));

        let rql_statement = "out(name,(Grayson,Lucas))".to_owned();
        assert!(rql_validator.is_valid(rql_statement));

        let rql_statement = "and(eq(name,John),eq(surname,Smith))".to_owned();
        assert!(rql_validator.is_valid(rql_statement));

        let rql_statement = "or(eq(login,congrate),eq(name,John))".to_owned();
        assert!(rql_validator.is_valid(rql_statement));

        let rql_statement = "not(eq(id,1))".to_owned();
        assert!(rql_validator.is_valid(rql_statement));

        let rql_statement = "eqf(isActive)".to_owned();
        assert!(rql_validator.is_valid(rql_statement));

        let rql_statement = "eqt(isActive)".to_owned();
        assert!(rql_validator.is_valid(rql_statement));

        let rql_statement = "eqn(name)".to_owned();
        assert!(rql_validator.is_valid(rql_statement));

        let rql_statement = "ie(name)".to_owned();
        assert!(rql_validator.is_valid(rql_statement));

        let rql_statement = "eq(name,John)".to_owned();
        assert!(rql_validator.is_valid(rql_statement));

        let rql_statement = "ge(salary,500)".to_owned();
        assert!(rql_validator.is_valid(rql_statement));

        let rql_statement = "gt(salary,600)".to_owned();
        assert!(rql_validator.is_valid(rql_statement));

        let rql_statement = "le(salary,1000)".to_owned();
        assert!(rql_validator.is_valid(rql_statement));

        let rql_statement = "lt(salary,900)".to_owned();
        assert!(rql_validator.is_valid(rql_statement));

        let rql_statement = "ne(name,Jackson)".to_owned();
        assert!(rql_validator.is_valid(rql_statement));
    }

    #[test]
    fn test_not_valid() {
        fn print_errors(error_message: &str) {
            eprintln!("{}", error_message);
        }

        let rql_validator = RqlValidator::new(Some(print_errors));

        let rql_statement = "in(name,John,Jackson,Liam)".to_owned();
        assert!(!rql_validator.is_valid(rql_statement));

        let rql_statement = "in(name,())".to_owned();
        assert!(!rql_validator.is_valid(rql_statement));

        let rql_statement = "nonexistent(name)".to_owned();
        assert!(!rql_validator.is_valid(rql_statement));

        let rql_statement = "and(name,Smith)".to_owned();
        assert!(!rql_validator.is_valid(rql_statement));

        let rql_statement = "and(eq(name,John),eq(surname,Smith),eq(login,congrate))".to_owned();
        assert!(!rql_validator.is_valid(rql_statement));

        let rql_statement = "not(eq(login,congrate),eq(name,John))".to_owned();
        assert!(!rql_validator.is_valid(rql_statement));

        let rql_statement = "eqf()".to_owned();
        assert!(!rql_validator.is_valid(rql_statement));

        let rql_statement = "eqf(eq(name,John))".to_owned();
        assert!(!rql_validator.is_valid(rql_statement));

        let rql_statement = "eqf(isActive,isProcessable)".to_owned();
        assert!(!rql_validator.is_valid(rql_statement));
    }
}
