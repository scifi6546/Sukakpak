mod allocators;
pub struct VertexBufferPool {}
impl VertexBufferPool {
    pub fn new(alloc_size: usize) -> Self {
        todo!()
    }
}
pub struct VertexBufferAllocation {}
trait AllocTarget {}
trait AllocateTarget {
    type SubTarget;
    /// allocates a block with a new size. If there is an existing block the data is copied over
    /// from the old block to a bock with the new size
    fn alloc_block(&mut self, size: usize);
    /// gets an offset block
    fn offset(&mut self, offset: usize) -> Self::SubTarget;
}
