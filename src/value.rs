#[derive(Debug)]

pub enum Value<'s> {
    Integer(i64),
    Number(f64),
    String(&'s str),
    Symbol(&'s str),
    Cons(&'s Value<'s>, &'s Value<'s>),
}
