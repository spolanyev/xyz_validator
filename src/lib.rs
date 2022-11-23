//!  # Examples
//!
//!  Basic usage:
//!
//!  ```
//! use xyz_validator::{RqlValidator, ValidatorInterface};
//!
//!fn main() {
//!    let rql_statement = "or(and(eq(name,John),eq(surname,Doe)),eq(surname,Smith))".to_owned();
//!
//!    //Check if RQL statement is valid
//!    let rql_validator: Box<dyn ValidatorInterface> = Box::new(RqlValidator::new());
//!    assert!(rql_validator.is_valid(rql_statement));
//!}
//!  ```

pub use self::interfaces::validator_interface::ValidatorInterface;
pub use self::rql_validator::RqlValidator;
pub mod interfaces;
pub mod rql_validator;
