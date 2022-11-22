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

    fn get_operators(&self, string: String) -> Vec<(String, Option<String>)> {
        let closing_parts = HashMap::from([('(', ')')]);
        let opening_parts = HashMap::from([(')', '(')]);

        let mut result: Vec<(String, Option<String>)> = vec![];

        let mut is_inside_parentheses = false;
        let mut operator = "".to_owned();
        let mut operator_content = "".to_owned();

        for char in string.chars() {
            //opening detected
            if closing_parts.get(&char).is_some() {
                is_inside_parentheses = true;
                println!("operator: {}", operator.to_lowercase());
                result.push((operator, None));
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
                operator = "".to_owned();
                operator_content.pop();

                if 0 == operator_content.len() {
                    continue;
                }
                println!("operator content: {}", operator_content);
                let last = result
                    .last_mut()
                    .expect("We have at least operator before the value");
                *last = (last.0.clone(), Some(operator_content));
                operator_content = "".to_owned();
            }
        }

        println!("{:#?}", result);
        result
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
        let expected = vec![("eq".to_owned(), Some("name,John".to_owned()))];
        assert_eq!(expected, rql_validator.get_operators(rql_statement));

        let rql_statement = "or(and(eq(name,John),eq(surname,Smith)),eq(surname,Doe))".to_owned();
        let expected = vec![
            ("or".to_owned(), None),
            ("and".to_owned(), None),
            ("eq".to_owned(), Some("name,John".to_owned())),
            ("eq".to_owned(), Some("surname,Smith".to_owned())),
            ("eq".to_owned(), Some("surname,Doe".to_owned())),
        ];
        assert_eq!(expected, rql_validator.get_operators(rql_statement));

        let rql_statement = "not(in(name,(John,Jackson,Liam)))".to_owned();
        let expected = vec![
            ("not".to_owned(), None),
            ("in".to_owned(), None),
            ("name".to_owned(), Some("John,Jackson,Liam".to_owned())),
        ];
        assert_eq!(expected, rql_validator.get_operators(rql_statement));
    }
}
