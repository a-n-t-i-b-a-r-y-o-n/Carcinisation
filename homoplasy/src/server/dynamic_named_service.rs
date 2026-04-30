/// Named service without the `const`
pub trait DynamicNamedService {
    fn name(&self) -> &str;
}