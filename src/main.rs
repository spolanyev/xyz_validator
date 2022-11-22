//@author Stanislav Polaniev <spolanyev@gmail.com>

use xyz_validator::RqlValidator;
use xyz_validator::ValidatorInterface;

fn main() {
    let rql_validator: Box<dyn ValidatorInterface> = Box::new(RqlValidator::new());

    let rql_statement = "or(and(eq(name,John),eq(surname,Smith)),eq(surname,Doe))".to_owned();
    assert!(rql_validator.is_valid(rql_statement));
}
