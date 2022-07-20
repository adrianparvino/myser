use crate::value::Value;
use core::mem::MaybeUninit;
use core::cell::{Cell, UnsafeCell};
use core::ops::Deref;
use core::ptr;
use core::fmt;

pub struct ValueCell<'s> {
    cell: UnsafeCell<MaybeUninit<Value<'s>>>,
    rc: Cell<usize>
}

pub struct RcValue<'s>(*const ValueCell<'s>);
impl<'s> Clone for RcValue<'s> {
    fn clone(&self) -> Self {
        let inner = unsafe { &*self.0 };
        inner.rc.set(inner.rc.get() + 1);

        RcValue(self.0)
    }
}
impl<'s> Drop for RcValue<'s> {
    fn drop(&mut self) {
        unsafe {
            let inner = &*self.0;
            inner.rc.set(inner.rc.get() - 1);

            if inner.rc.get() > 0 {
                return;
            }

            ptr::drop_in_place((&mut *inner.cell.get()).assume_init_mut());
        }
    }
}
impl<'s> Deref for RcValue<'s> {
    type Target = Value<'s>;

    fn deref(&self) -> &Self::Target {
        unsafe { (*(*self.0).cell.get()).assume_init_ref() }
    }
}
impl<'s> fmt::Debug for RcValue<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.deref().fmt(f)
    }
}


pub struct Pool<'s, const N: usize> {
    pool: [ValueCell<'s>; N],
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
            if cell.rc.get() > 0 { continue; }

            cell.rc.set(1);
            unsafe {
                cell.cell.get().write(MaybeUninit::new(value));
            }

            return Ok(RcValue(cell));
        }

        for cell in self.pool[0..n].iter() {
            if cell.rc.get() > 0 { continue; }

            cell.rc.set(1);
            unsafe {
                cell.cell.get().write(MaybeUninit::new(value));
            }

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
