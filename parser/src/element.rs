use crate::ElementMetadata;

#[derive(Debug, PartialEq)]
pub struct Element<T> {
    pub value: T,
    pub metadata: ElementMetadata,
}

impl<T> Element<T> {
    pub fn value(&self) -> &T {
        &self.value
    }
}
