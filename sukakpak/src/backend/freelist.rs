use std::collections::HashMap;
pub type RenderpassID = usize;
#[derive(Default)]
pub struct Freelist<T> {
    to_free: HashMap<T, RenderpassID>,
    by_renderpass: HashMap<RenderpassID, Vec<T>>,
}
impl<T> Freelist {
    /// Marks the item as used
    pub fn push(&mut self, item: T, renderpass: RenderpassID) {
        if self.by_renderpass.contains_key(&renderpass) {
            self.by_renderpass.get_mut(&renderpass).unwrap().push(item);
        } else {
            self.by_renderpass.insert(renderpass, vec![item]);
        }
    }
    /// Frees a component
    pub fn try_free() {}
}
