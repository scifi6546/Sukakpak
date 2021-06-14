use super::Allocateable;
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
pub struct BuddyAllocator<T: Allocateable> {
    free_tree: BinaryTree,
    memory: T,
    //these should be constant for the life of the allocator
    block_levels: usize,
    //size of a block in powers of 2
    block_power: usize,
}
#[derive(Clone)]
pub struct Allocation<T> {
    pub data: T,
    alloc_index: usize,
}
impl<T: Allocateable> BuddyAllocator<T> {
    fn block_size(&self) -> usize {
        1 << self.block_power
    }
    pub fn new(block_power: usize, block_levels: usize, mut memory: T) -> Self {
        let num_blocks = 1 << block_levels;
        let memory_size = (1 << block_power) * num_blocks;
        memory.realloc(memory_size);
        Self {
            memory,
            free_tree: BinaryTree::Data(Free::Free),
            block_power,
            block_levels,
        }
    }
    pub fn alloc(&mut self, allocation_size: usize) -> Option<Allocation<T::SubMemory>> {
        let depth_in_tree = self.block_levels - self.get_block_level(allocation_size);
        if let Some(alloc_index) = self.free_tree.get_first_free(depth_in_tree) {
            let mem_index = self.get_alloc_memory_index(alloc_index);

            let data = self.memory.offset(mem_index);
            Some(Allocation { data, alloc_index })
        } else {
            None
        }
    }
    pub fn free(&mut self, allocation: Allocation<T::SubMemory>) {
        self.free_tree.free(allocation.alloc_index);
    }
    fn get_alloc_memory_index(&self, alloc_index: usize) -> usize {
        (0..self.block_levels)
            .map(|i| ((alloc_index >> i) & 1) << self.block_power)
            .sum::<usize>()
    }
    fn get_block_level(&self, alloc_size: usize) -> usize {
        let num_blocks = alloc_size / self.block_size()
            + if alloc_size % self.block_size() == 1 {
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
    struct Memory {
        data: Vec<u8>,
    }
    impl Memory {
        pub fn new() -> Self {
            Self { data: vec![] }
        }
    }
    impl Allocateable for Memory {
        type SubMemory = Vec<u8>;
        /// reallocates self memory to new size. copies over old memory
        fn realloc(&mut self, new_size: usize) {
            self.data.resize(new_size, 0);
        }
        /// allocates an offset of memory
        fn offset(&mut self, offset: usize) -> Self::SubMemory {
            let new_size = self.data.len() - offset;
            (0..new_size).map(|i| self.data[i + offset]).collect()
        }
    }
    use super::*;
    #[test]
    fn build() {
        let _tree = BuddyAllocator::new(8, 4, Memory::new());
    }
    #[test]
    fn allocate() {
        let mut tree = BuddyAllocator::new(8, 4, Memory::new());
        let _alloc = tree.alloc(10).unwrap();
    }
    #[test]
    fn dealloc() {
        let mut tree = BuddyAllocator::new(8, 4, Memory::new());
        let alloc = tree.alloc(10).unwrap();
        tree.free(alloc);
    }
    #[test]
    fn alloc_batches() {
        let alloc_list = [100, 2341, 213, 1234, 1234, 12];
        let mut tree = BuddyAllocator::new(8, 8, Memory::new());
        let mut allocs = alloc_list
            .iter()
            .map(|i| tree.alloc(*i).unwrap())
            .collect::<Vec<_>>();
        allocs
            .drain(..)
            .map(|alloc| tree.free(alloc))
            .collect::<Vec<_>>();
    }
}
