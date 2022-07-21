use crate::{ancestory::Ancestory, parser::Parser, ElementMetadata, Status};

trait ElementParser: Parser {
    fn init(&mut self, metadata: &ElementMetadata, max_size: u64) -> Status;
    fn init_after_seek(&mut self, child_ancestory: &Ancestory, child_metadata: &ElementMetadata) {
        unreachable!()
    }
    fn get_cached_metadata(&self) -> Option<ElementMetadata>;
    fn was_skipped(&self) -> bool {
        false
    }
}
