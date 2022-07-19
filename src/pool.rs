use crate::{constants::{
    NIL_SYM, QUOTE_SYM, WHILE_SYM, ZERO
}, value::Value};
use core::sync::atomic::{AtomicUsize, Ordering};
use core::mem::MaybeUninit;
use core::cell::UnsafeCell;

pub struct Pool<'s, const N: usize> {
    pool: [UnsafeCell<MaybeUninit<Value<'s>>>; N],
    alloced: AtomicUsize,
}

unsafe impl<'s, const N: usize> Sync for Pool<'s, N> {}

impl<'s, const N: usize> Pool<'s, N> {
    pub const fn new() -> Self {
        Pool {
            pool: unsafe { MaybeUninit::uninit().assume_init() },
            alloced: AtomicUsize::new(0),
        }
    }

    fn alloc(&self, t: Value<'s>) -> Result<&mut Value<'s>, Value<'s>> {
        match self.pool.get(self.alloced.fetch_add(1, Ordering::AcqRel)) {
            Some(ptr) => {
                Ok(unsafe { (&mut *ptr.get()).write(t) })
            }
            None => return Err(t),
        }
    }

    pub fn new_integer(&self, n: i64) -> &Value<'s> {
        match n {
            0 => &ZERO,
            _ => self.alloc(Value::Integer(n)).unwrap()
        }
    }

    pub fn new_symbol(&self, symbol: &'s str) -> &Value<'s> {
        match symbol {
            "nil" => &NIL_SYM,
            "while" => &WHILE_SYM,
            "quote" => &QUOTE_SYM,
            _ => self.alloc(Value::Symbol(symbol)).unwrap()
        }
    }

    pub fn new_cons(&self, car: &'s Value<'s>, cdr: &'s Value<'s>) -> &Value<'s> {
        self.alloc(Value::Cons(car, cdr)).unwrap()
    }

    pub fn used(&self) -> usize {
        self.alloced.load(Ordering::Acquire)
    }
}
