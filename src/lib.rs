#[cfg(feature = "ring")]
pub mod ring;

pub mod simple;

trait Buffer<T> {
    /// Appends the given value to the buffer and returns the new size of the buffer.
    fn append(&mut self, value: Vec<T>) -> usize;

    /// Gets the current contents of the buffer and clears the buffer.
    fn get_and_clear(&mut self) -> Vec<T>;
}
