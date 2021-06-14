pub mod buddy;
pub mod resizable_buddy;
pub trait Allocateable {
    /// sub memory allocated form main pool
    type SubMemory;
    /// reallocates self memory to new size. copies over old memory
    fn realloc(&mut self, new_size: usize);
    /// allocates an offset of memory
    fn offset(&mut self, offset: usize) -> Self::SubMemory;
}
