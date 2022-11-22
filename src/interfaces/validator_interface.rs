//@author Stanislav Polaniev <spolanyev@gmail.com>

pub trait ValidatorInterface {
    fn is_valid(&self, data: String) -> bool;
}
