use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};
pub type RenderpassID = usize;
#[derive(Default)]
pub struct Freelist<T: Eq + Hash + Clone> {
    too_free: HashSet<T>,
    by_renderpass: HashMap<RenderpassID, Vec<T>>,
}
impl<T: Eq + Hash + Clone> Freelist<T> {
    /// Marks the item as used
    pub fn push(&mut self, item: T, renderpass: RenderpassID) {
        if self.by_renderpass.contains_key(&renderpass) {
            self.by_renderpass.get_mut(&renderpass).unwrap().push(item);
        } else {
            self.by_renderpass.insert(renderpass, vec![item]);
        }
    }
    /// Marks a component as to be freed
    pub fn try_free(&mut self, item: T) {
        self.too_free.insert(item);
    }
    /// Returns all items to free in a renderpass
    pub fn finish_renderpass(&mut self, done_renderpass: RenderpassID) -> HashSet<T> {
        let mut out_free = self.too_free.clone();
        for (rendeprass_id, item_vec) in self.by_renderpass.iter() {
            if rendeprass_id != &done_renderpass {
                for item in item_vec.iter() {
                    out_free.remove(item);
                }
            }
        }
        for freed in out_free.iter() {
            self.too_free.remove(freed);
        }
        self.by_renderpass.remove(&done_renderpass);
        return out_free;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn build_freelist() {
        let _list = Freelist::<u32>::default();
    }
    #[test]
    fn run_simple_render() {
        let mut list: Freelist<u32> = Default::default();
        list.push(1, 0);
        let r = list
            .finish_renderpass(0)
            .iter()
            .copied()
            .collect::<Vec<_>>();
        assert_eq!(r.len(), 0);
        list.try_free(1);
        let r2 = list
            .finish_renderpass(0)
            .iter()
            .copied()
            .collect::<Vec<_>>();
        assert_eq!(r2, vec![1]);
    }
    #[test]
    fn multiple_renders() {
        let mut list: Freelist<u32> = Default::default();
        list.push(1, 0);
        list.push(1, 1);
        let r = list
            .finish_renderpass(0)
            .iter()
            .copied()
            .collect::<Vec<_>>();
        assert_eq!(r.len(), 0);
        list.try_free(1);
        let r2 = list
            .finish_renderpass(0)
            .iter()
            .copied()
            .collect::<Vec<_>>();
        assert_eq!(r2.len(), 0);

        let r3 = list
            .finish_renderpass(1)
            .iter()
            .copied()
            .collect::<Vec<_>>();

        assert_eq!(r3, vec![1]);
    }
}
