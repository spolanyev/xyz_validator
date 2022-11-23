//!  # Examples
//!
//!  Basic usage:
//!
//!  ```
//! use xyz_validator::{RqlValidator, ValidatorInterface};
//!
//! fn main() {
//!     //Check if RQL statement is valid
//!     let rql_validator: Box<dyn ValidatorInterface> = Box::new(RqlValidator::new());
//!
//!     let rql_statement = "or(and(eq(name,John),eq(surname,Smith)),eq(surname,Doe))".to_owned();
//!     assert!(rql_validator.is_valid(rql_statement));
//! }
//!  ```

pub use self::interfaces::validator_interface::ValidatorInterface;
pub use self::rql_validator::RqlValidator;
pub mod interfaces;
pub mod rql_validator;
