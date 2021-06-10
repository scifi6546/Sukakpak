enum BinaryTree {
    Data(Free),
    Root(Box<[BinaryTree; 2]>),
}
impl BinaryTree {
    /// Gets first element with free root and sets it to used
    pub fn get_first_free(&mut self, traverse_levels: usize) -> Option<usize> {
        match self {
            Self::Data(free) => {
                if traverse_levels == 0 {
                    if *free == Free::Free {
                        *free = Free::Used;
                        Some(0)
                    } else {
                        None
                    }
                } else {
                    match free {
                        Free::Free => {
                            *self = BinaryTree::Root(Box::new([
                                BinaryTree::Data(Free::Free),
                                BinaryTree::Data(Free::Free),
                            ]));
                            match self {
                                BinaryTree::Data(_) => panic!("impossible condition"),
                                BinaryTree::Root(data) => {
                                    Some(data[0].get_first_free(traverse_levels - 1).unwrap() << 1)
                                }
                            }
                        }
                        Free::Used => None,
                    }
                }
            }
            Self::Root(data) => {
                if traverse_levels > 0 {
                    if let Some(first_try) = data[0].get_first_free(traverse_levels - 1) {
                        Some(first_try << 1)
                    } else if let Some(second_try) = data[1].get_first_free(traverse_levels - 1) {
                        Some((second_try << 1) + 1)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
    }
    pub fn free(&mut self, index: usize) {
        match self {
            Self::Data(free) => *free = Free::Free,
            Self::Root(trees) => trees[index & 1].free(index >> 1),
        }
    }
}
#[derive(Copy, Clone, PartialEq)]
enum Free {
    Used,
    Free,
}
pub struct BuddyAllocator {
    free_tree: BinaryTree,
    data: [u8; Self::BLOCK_LEVELS * Self::MIN_BLOCK_SIZE],
}
#[derive(Clone)]
pub struct Allocation {
    pub data: *mut u8,
    alloc_index: usize,
}
impl BuddyAllocator {
    const STARTING_BLOCK_POW: usize = 16;
    const MIN_BLOCK_SIZE: usize = 1 << Self::STARTING_BLOCK_POW;
    const BLOCK_LEVELS: usize = 8;
    pub fn new() -> Self {
        Self {
            free_tree: BinaryTree::Data(Free::Free),
            data: [0; Self::BLOCK_LEVELS * Self::MIN_BLOCK_SIZE],
        }
    }
    pub fn alloc(&mut self, allocation_size: usize) -> Option<Allocation> {
        let depth_in_tree = Self::BLOCK_LEVELS - Self::get_block_level(allocation_size);
        if let Some(alloc_index) = self.free_tree.get_first_free(depth_in_tree) {
            let mem_index = Self::get_alloc_memory_index(alloc_index);
            let data = unsafe { self.data.as_ptr().offset(mem_index as isize) as *mut u8 };
            Some(Allocation { data, alloc_index })
        } else {
            None
        }
    }
    pub fn free(&mut self, allocation: Allocation) {
        self.free_tree.free(allocation.alloc_index);
    }
    fn get_alloc_memory_index(alloc_index: usize) -> usize {
        (0..Self::BLOCK_LEVELS)
            .map(|i| ((alloc_index >> i) & 1) << Self::STARTING_BLOCK_POW)
            .sum::<usize>()
    }
    fn get_block_level(alloc_size: usize) -> usize {
        let num_blocks = alloc_size / Self::MIN_BLOCK_SIZE
            + if alloc_size % Self::MIN_BLOCK_SIZE == 1 {
                1
            } else {
                0
            };
        let mut max_size = 0;
        for i in 0..std::mem::size_of::<usize>() {
            if (num_blocks >> i) & 1 == 1 {
                max_size = i;
            };
        }
        return max_size;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn build() {
        let _tree = BuddyAllocator::new();
    }
    #[test]
    fn allocate() {
        let mut tree = BuddyAllocator::new();
        let _alloc = tree.alloc(10).unwrap();
    }
    #[test]
    fn dealloc() {
        let mut tree = BuddyAllocator::new();
        let alloc = tree.alloc(10).unwrap();
        tree.free(alloc);
    }
    #[test]
    fn alloc_batches() {
        let alloc_list = [100, 2341, 213, 1234, 1234, 12];
        let mut tree = BuddyAllocator::new();
        let mut allocs = alloc_list
            .iter()
            .map(|i| tree.alloc(*i).unwrap())
            .collect::<Vec<_>>();
        allocs
            .drain(..)
            .map(|alloc| tree.free(alloc))
            .collect::<Vec<_>>();
    }
    #[test]
    fn test_block_level() {
        assert_eq!(BuddyAllocator::get_block_level(4), 0);
        assert_eq!(
            BuddyAllocator::get_block_level(BuddyAllocator::MIN_BLOCK_SIZE + 1),
            1
        );
        assert_eq!(
            BuddyAllocator::get_block_level(BuddyAllocator::MIN_BLOCK_SIZE * 3 + 1),
            2
        );
    }
}
