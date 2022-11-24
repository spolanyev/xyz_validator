//!  # Examples
//!
//!  Basic usage:
//!
//!  ```
//!use xyz_validator::{RqlValidator, ValidatorInterface};
//!
//!fn main() {
//!    //test a valid rql
//!    let valid_rql_statement = "or(and(eq(name,John),eq(surname,Doe)),eq(surname,Smith))".to_owned();
//!    let rql_validator: Box<dyn ValidatorInterface> = Box::new(RqlValidator::new(None));
//!    assert!(rql_validator.is_valid(valid_rql_statement));
//!
//!    //test an invalid rql, we want to see errors
//!    let invalid_rql_statement = "and(eq(name,John),eq(surname,Doe),eq(surname,Smith))".to_owned();
//!
//!    //define a callback function to do something with an error message - here we print the error message to stderr
//!    fn print_errors(error_message: &str) {
//!        eprintln!("{}", error_message);
//!    }
//!
//!    let rql_validator: Box<dyn ValidatorInterface> =
//!        Box::new(RqlValidator::new(Some(print_errors)));
//!    //It will print "Node 'and' should have 2 nested nodes" to stderr
//!    assert!(!rql_validator.is_valid(invalid_rql_statement));
//!}
//!  ```

pub use self::interfaces::validator_interface::ValidatorInterface;
pub use self::rql_validator::RqlValidator;
pub mod interfaces;
pub mod rql_validator;
