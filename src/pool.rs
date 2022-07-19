use crate::{value::Value};
use core::sync::atomic::{AtomicIsize, Ordering};
use core::mem::MaybeUninit;
use core::cell::UnsafeCell;
use core::ops::Deref;
use core::fmt;

pub struct ValueCell<'s> {
    cell: MaybeUninit<Value<'s>>,
    rc: usize
}

pub struct RcValue<'s>(*mut ValueCell<'s>);
impl<'s> Clone for RcValue<'s> {
    fn clone(&self) -> Self {
        let inner = unsafe { &mut *self.0 };
        inner.rc += 1;

        RcValue(self.0)
    }
}
impl<'s> Drop for RcValue<'s> {
    fn drop(&mut self) {
        unsafe {
            let inner = &mut *self.0;
            inner.rc -= 1;

            if inner.rc > 0 {
                return;
            }

            std::mem::replace(&mut inner.cell, MaybeUninit::uninit()).assume_init();
        }
    }
}
impl<'s> Deref for RcValue<'s> {
    type Target = Value<'s>;

    fn deref(&self) -> &Self::Target {
        unsafe { (*self.0).cell.assume_init_ref() }
    }
}
impl<'s> fmt::Debug for RcValue<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.deref().fmt(f)
    }
}


pub struct Pool<'s, const N: usize> {
    pool: [UnsafeCell<ValueCell<'s>>; N],
    alloced: UnsafeCell<usize>,
}

unsafe impl<'s, const N: usize> Sync for Pool<'s, N> {}

impl<'s, const N: usize> Pool<'s, N> {
    pub fn new() -> Self {
        Pool {
            pool: unsafe { MaybeUninit::zeroed().assume_init() },
            alloced: UnsafeCell::new(0)
        }
    }

    fn alloc(&self, value: Value<'s>) -> Result<RcValue<'s>, Value<'s>> {
        let n: usize = unsafe {
            self.alloced.get().write(*self.alloced.get() + 1);
            *self.alloced.get() % N
        };

        for cell in self.pool[n..N].iter() {
            let cell_ptr = cell.get();
            let mut cell = unsafe { &mut *cell_ptr };

            if cell.rc > 0 { continue; }

            cell.rc = 1;
            cell.cell = MaybeUninit::new(value);

            return Ok(RcValue(cell_ptr));
        }

        for cell in self.pool[0..n].iter() {
            let cell_ptr = cell.get();
            let mut cell = unsafe { &mut *cell_ptr };

            if cell.rc > 0 { continue; }

            cell.rc = 1;
            cell.cell = MaybeUninit::new(value);

            return Ok(RcValue(cell_ptr));
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
