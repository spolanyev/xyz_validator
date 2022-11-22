use crate::interfaces::validator_interface::ValidatorInterface;
use std::collections::HashMap;

pub struct RqlValidator {}

impl ValidatorInterface for RqlValidator {
    fn is_valid(&self, data: String) -> bool {
        if !self.is_parentheses_matched(data) {
            return false;
        }

        true
    }
}

impl RqlValidator {
    fn new() -> Self {
        Self {}
    }

    fn is_parentheses_matched(&self, string: String) -> bool {
        let closing_parts = HashMap::from([('(', ')')]);
        let opening_parts = HashMap::from([(')', '(')]);

        let mut stack = vec![];

        for char in string.chars() {
            //opening detected
            if let Some(closing_part) = closing_parts.get(&char) {
                stack.push(closing_part);
                continue;
            }

            //closing detected
            if opening_parts.get(&char).is_some() {
                if stack.pop() != Some(&char) {
                    return false;
                }
            }
        }

        if stack.is_empty() {
            return true;
        }

        false
    }

    fn get_operators(&self, string: String) -> Vec<(String, Option<String>, usize)> {
        let closing_parts = HashMap::from([('(', ')')]);
        let opening_parts = HashMap::from([(')', '(')]);

        let mut result: Vec<(String, Option<String>, usize)> = vec![];

        let mut is_inside_parentheses = false;
        let mut operator = "".to_owned();
        let mut operator_content = "".to_owned();
        let mut level: usize = 0;

        for char in string.chars() {
            //opening detected
            if closing_parts.get(&char).is_some() {
                level += 1;
                is_inside_parentheses = true;
                println!("operator: {}", operator.to_lowercase());
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
            if opening_parts.get(&char).is_some() {
                level -= 1;
                operator = "".to_owned();
                operator_content.pop();

                if 0 == operator_content.len() {
                    continue;
                }
                println!("operator content: {}", operator_content);
                let last = result
                    .last_mut()
                    .expect("We have at least operator before the value");
                *last = (last.0.clone(), Some(operator_content), last.2);
                operator_content = "".to_owned();
            }
        }

        println!("{:#?}", result);
        result
    }

    fn is_operators_valid(&self, operators: Vec<(String, Option<String>, usize)>) -> bool {
        true
    }

    fn add_nested_nodes_quantity(
        &self,
        operators: Vec<(String, Option<String>, usize)>,
    ) -> Vec<(String, Option<String>, usize, usize)> {
        operators
            .iter()
            .map(|x: &(String, Option<String>, usize)| {
                return if x.1.is_none() {
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
                };
            })
            .collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn check_parentheses() {
        let rql_validator = RqlValidator::new();

        let rql_statement = "eq(name,John)".to_owned();
        assert!(rql_validator.is_valid(rql_statement));

        let rql_statement = "and(eq(name,John),eq(surname,Smith))".to_owned();
        assert!(rql_validator.is_valid(rql_statement));

        let rql_statement = "or(and(eq(name,John),eq(surname,Smith)),eq(surname,Doe))".to_owned();
        assert!(rql_validator.is_valid(rql_statement));
    }

    #[test]
    fn get_operators() {
        let rql_validator = RqlValidator::new();

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
        let rql_validator = RqlValidator::new();

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
}
