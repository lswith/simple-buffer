use std::sync::{Arc, Mutex};

use ringbuf::HeapRb;
use ringbuf::Rb;

#[derive(Clone)]
pub struct Buffer<T> {
    data: Arc<Mutex<HeapRb<T>>>,
}


impl<T> Buffer<T> {
    pub fn new(size: usize) -> Self {
        Buffer {
            data: Arc::new(Mutex::new(HeapRb::<T>::new(size))),
        }
    }

    #[allow(unused_mut)]
    pub fn append(&mut self, mut value: Vec<T>) -> usize {
        let mut lock = self.data.lock().unwrap();
        lock.push_iter_overwrite(value.into_iter());
        lock.len()
    }

    pub fn get_and_clear(&mut self) -> Vec<T> {
        let mut lock = self.data.lock().unwrap();
        lock.pop_iter().collect()
    }
}

#[cfg(test)]
mod test {
    use std::fmt::Display;

    use super::Buffer;

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
        buf.append(b)
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
        let buf = Buffer::<TestStruct>::new(100000);
        let mut handles = tokio::task::JoinSet::new();
        for _ in 0..100 {
            handles.spawn(spawn_when_full(buf.clone(), 100000, 100));
        }
        while handles.join_next().await.is_some() {}
    }

}
