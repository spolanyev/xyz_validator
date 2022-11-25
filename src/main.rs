//@author Stanislav Polaniev <spolanyev@gmail.com>

use xyz_validator::{RqlValidator, ValidatorInterface};

fn main() {
    let valid_rql_statement = "or(and(eq(name,John),eq(surname,Doe)),eq(surname,Smith))".to_owned();
    let rql_validator: Box<dyn ValidatorInterface> = Box::new(RqlValidator::new(None));
    assert!(rql_validator.is_valid(valid_rql_statement));

    //to view errors we should define a callback function for `String` argument
    fn your_handle_error_function(your_var: String) {
        eprintln!("{}", your_var);
    }

    let rql_validator: Box<dyn ValidatorInterface> =
        Box::new(RqlValidator::new(Some(your_handle_error_function)));

    let invalid_rql_statement = "and(eq(name,John),eq(surname,Doe),eq(surname,Smith))".to_owned();
    assert!(!rql_validator.is_valid(invalid_rql_statement));
    //"Node `and` should have 2 nested nodes"
}
