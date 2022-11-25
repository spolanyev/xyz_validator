//@author Stanislav Polaniev <spolanyev@gmail.com>

use xyz_validator::{RqlValidator, ValidatorInterface};

fn main() {
    let valid_rql_statement = "or(and(eq(name,John),eq(surname,Doe)),eq(surname,Smith))".to_owned();
    let rql_validator = RqlValidator::new(None);
    assert!(rql_validator.is_valid(valid_rql_statement));

    //to view an error we should define a callback function

    //let's print an error to stderr
    fn print_error(error_message: &str) {
        eprintln!("{}", error_message);
    }

    let rql_validator = RqlValidator::new(Some(print_error));

    let invalid_rql_statement = "and(eq(name,John),eq(surname,Doe),eq(surname,Smith))".to_owned();
    assert!(!rql_validator.is_valid(invalid_rql_statement));
    //"Node `and` should have 2 nested nodes"
}
