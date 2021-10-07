pub enum RefrenceStatus {
    InUse,
    NotUsed,
}
/// Keeps refrence count of data and signals if refrence count reaches zero
pub struct RefCounter<T> {
    counter: usize,
    data: T,
}
impl<T> RefCounter<T> {
    pub fn new(data: T, num_refs: usize) -> Self {
        Self {
            counter: num_refs,
            data,
        }
    }
    pub fn get(&self) -> &T {
        &self.data
    }
    pub fn refrences(&self) -> usize {
        self.counter
    }
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.data
    }
    /// Gets data, consumes self
    pub fn drain(self) -> T {
        self.data
    }
    pub fn incr_refrence(&mut self) -> RefrenceStatus {
        self.counter += 1;
        RefrenceStatus::InUse
    }
    pub fn decr_refrence(&mut self) -> RefrenceStatus {
        // invalid state if number of refrences becomes negative
        assert!(self.counter >= 1);
        self.counter -= 1;
        match self.counter {
            0 => RefrenceStatus::NotUsed,
            _ => RefrenceStatus::InUse,
        }
    }
}
