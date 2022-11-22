pub(crate) trait ValidatorInterface {
    fn is_valid(&self, data: String) -> bool;
}
