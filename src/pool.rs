use crate::{value::Value};
use core::sync::atomic::{AtomicIsize, Ordering};
use core::mem::MaybeUninit;
use core::cell::UnsafeCell;
use core::ops::Deref;
use core::fmt;

pub struct ValueCell<'s> {
    cell: UnsafeCell<MaybeUninit<Value<'s>>>,
    rc: AtomicIsize
}

pub struct RcValue<'s>(*mut ValueCell<'s>);
impl<'s> Clone for RcValue<'s> {
    fn clone(&self) -> Self {
        let inner = unsafe { &mut *self.0 };
        inner.rc.fetch_add(1, Ordering::AcqRel);

        RcValue(self.0)
    }
}
impl<'s> Drop for RcValue<'s> {
    fn drop(&mut self) {
        let inner = unsafe { &mut *self.0 };
        if inner.rc.fetch_sub(1, Ordering::Release) != 1 {
            return;
        }

        let _ = unsafe { inner.cell.get().replace(MaybeUninit::uninit()).assume_init() };
    }
}
impl<'s> Deref for RcValue<'s> {
    type Target = Value<'s>;

    fn deref(&self) -> &Self::Target {
        unsafe { (&mut *(*self.0).cell.get()).assume_init_ref() }
    }
}
impl<'s> fmt::Debug for RcValue<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.deref().fmt(f)
    }
}


pub struct Pool<'s, const N: usize> {
    pool: [UnsafeCell<ValueCell<'s>>; N],
}

unsafe impl<'s, const N: usize> Sync for Pool<'s, N> {}

impl<'s, const N: usize> Pool<'s, N> {
    pub fn new() -> Self {
        Pool {
            pool: unsafe { MaybeUninit::zeroed().assume_init() },
        }
    }

    fn alloc(&self, value: Value<'s>) -> Result<RcValue<'s>, Value<'s>> {
        for cell in self.pool.iter() {
            let cell = unsafe { &mut *cell.get() };
            if cell.rc.fetch_add(1, Ordering::AcqRel) != 0 {
                cell.rc.fetch_sub(1, Ordering::AcqRel);
                continue;
            }

            unsafe { cell.cell.get().write(MaybeUninit::new(value)) }

            return Ok(RcValue(cell));
        }

        Err(value)
    }

    pub fn new_integer(&self, n: i64) -> RcValue<'s> {
        match n {
            // 0 => &ZERO,
            _ => self.alloc(Value::Integer(n)).unwrap()
        }
    }

    pub fn new_symbol(&self, symbol: &'s str) -> RcValue<'s> {
        match symbol {
            // "+" => &PLUS_SYM,
            // "-" => &MINUS_SYM,
            // "nil" => &NIL_SYM,
            // "while" => &WHILE_SYM,
            // "quote" => &QUOTE_SYM,
            _ => self.alloc(Value::Symbol(symbol)).unwrap()
        }
    }

    pub fn new_cons(&self, car: RcValue<'s>, cdr: RcValue<'s>) -> RcValue<'s> {
        self.alloc(Value::Cons(car, cdr)).unwrap()
    }

    // pub fn used(&self) -> usize {
    //     self.alloced.load(Ordering::Acquire)
    // }
}
