## Basic usage

```rust
use xyz_validator::{RqlValidator, ValidatorInterface};

fn main() {
    let rql_statement = "or(and(eq(name,John),eq(surname,Doe)),eq(surname,Smith))".to_owned();

    //Check if RQL statement is valid
    let rql_validator: Box<dyn ValidatorInterface> = Box::new(RqlValidator::new());
    assert!(rql_validator.is_valid(rql_statement));
}
```
