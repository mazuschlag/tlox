use std::convert::TryInto;

pub trait PoolRef {
    fn new(val: u32) -> Self;

    fn val(&self) -> u32;
}

#[derive(Debug)]
pub struct Pool<T>(Vec<T>);

impl<T> Pool<T> {
    pub fn default() -> Self {
        Self(Vec::new())
    }

    pub fn get(&self, pool_ref: impl PoolRef) -> &T {
        &self.0[pool_ref.val() as usize]
    }

    pub fn add<U: PoolRef>(&mut self, obj: T) -> U {
        let idx = self.0.len();
        self.0.push(obj);
        PoolRef::new(idx.try_into().expect(&format!("too many objects in the pool")))
    }
}

pub struct Pools<T, U>(pub Pool<T>, pub Pool<U>);