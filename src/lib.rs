use std::sync::{Arc, Mutex};

/// A simple buffer that can be used to store data from multiple threads.
/// NOTE: This buffer has no capacity limit and can grow indefinitely.
#[derive(Debug, Clone)]
pub struct Buffer<T> {
    data: Arc<Mutex<Vec<T>>>,
}

impl<T> Default for Buffer<T> {
    fn default() -> Self {
        Buffer {
            data: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl<T> Buffer<T> {
    /// Appends the given value to the buffer and returns the new size of the buffer.
    pub fn append(&mut self, value: &mut Vec<T>) -> usize {
        let mut lock = self.data.lock().unwrap();
        lock.append(value);
        lock.len()
    }

    /// Gets the current contents of the buffer and clears the buffer.
    pub fn get_and_clear(&mut self) -> Vec<T> {
        let mut lock = self.data.lock().unwrap();
        lock.drain(..).collect()
    }
}

#[cfg(test)]
mod test {
    use std::fmt::Display;

    use crate::Buffer;

    #[derive(Clone)]
    struct TestStruct {
        a: usize,
        b: usize,
    }

    impl Display for TestStruct {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "(a: {}, b: {})", self.a, self.b)
        }
    }

    async fn add_to_buffer(mut buf: Buffer<TestStruct>, size: usize) -> usize {
        let mut b = Vec::new();
        for i in 0..size {
            b.push(TestStruct { a: i, b: i });
        }
        buf.append(&mut b)
    }

    async fn read_from_buffer(mut buf: Buffer<TestStruct>) -> Vec<TestStruct> {
        buf.get_and_clear()
    }

    async fn spawn_when_full(buf: Buffer<TestStruct>, size: usize, chunks: usize) {
        let smaller = size / chunks;
        for _ in 0..chunks {
            let len = add_to_buffer(buf.clone(), smaller).await;
            if len == size {
                println!("adding {} to buffer", smaller);
                let _ = tokio::task::spawn(read_and_print(buf.clone())).await;
            }
        }
    }

    async fn read_and_print(buf: Buffer<TestStruct>) {
        let data = read_from_buffer(buf).await;
        println!("data size: {}", data.len());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_buffer_multi_write_100000_10() {
        let buf = Buffer::<TestStruct>::default();
        let mut handles = tokio::task::JoinSet::new();
        for _ in 0..100 {
            handles.spawn(spawn_when_full(buf.clone(), 100000, 100));
        }
        while handles.join_next().await.is_some() {}
    }

}
