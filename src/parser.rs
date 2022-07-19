use nom::{
    AsChar,
    IResult,
    InputTakeAtPosition,
    branch::alt,
    bytes::complete as bytes,
    character::complete as character,
    combinator::map,
};

use crate::value::Value;
use crate::pool::Pool;
use crate::constants::NIL_SYM;

pub fn integer<'s, const N: usize>(pool: &'s Pool<'s, N>, input: &'s str) -> IResult<&'s str, &'s Value<'s>> {
    map(character::i64, |n| pool.new_integer(n))(input)
}

pub fn cons_end<'s>(input: &'s str) -> IResult<&'s str, &'s Value<'s>> {
    let (input, _) = character::space0(input)?;
    let (input, _) = bytes::tag(")")(input)?;
    Ok((input, &NIL_SYM))
}

pub fn cons_pair<'s, const N: usize>(pool: &'s Pool<'s, N>, input: &'s str) -> IResult<&'s str, &'s Value<'s>> {
    let (input, _) = character::space0(input)?;
    let (input, _) = bytes::tag(".")(input)?;
    let (input, _) = character::space0(input)?;
    let (input, cdr) = parse(pool, input)?;
    let (input, _) = character::space0(input)?;
    let (input, _) = bytes::tag(")")(input)?;
    Ok((input, cdr))
}

pub fn cons_rest<'s, const N: usize>(pool: &'s Pool<'s, N>, input: &'s str) -> IResult<&'s str, &'s Value<'s>> {
    alt((
        cons_end,
        |input| cons_pair(pool, input),
        |input| {
            let (input, _) = character::space0(input)?;
            let (input, car) = parse(pool, input)?;
            let (input, cdr) = cons_rest(pool, input)?;

            Ok((input, pool.new_cons(car, cdr)))
        }
    ))(input)
}

pub fn cons<'s, const N: usize>(pool: &'s Pool<'s, N>, input: &'s str) -> IResult<&'s str, &'s Value<'s>> {
    let (input, _) = character::space0(input)?;
    let (input, _) = bytes::tag("(")(input)?;
    let (input, car) = parse(pool, input)?;
    let (input, cdr) = cons_rest(pool, input)?;

    Ok((input, pool.new_cons(car, cdr)))
}

pub fn symbol<'s, const N: usize>(pool: &'s Pool<'s, N>, input: &'s str) -> IResult<&'s str, &'s Value<'s>> {
    let (input, _) = character::space0(input)?;
    let (input, symbol) = input.split_at_position(
        |c| !c.is_alphanum() && c != '+' && c != '-'
    )?;
    Ok((input, pool.new_symbol(symbol)))
}

pub fn parse<'s, const N: usize>(pool: &'s Pool<'s, N>, input: &'s str) -> IResult<&'s str, &'s Value<'s>> {
    let (input, _) = character::space0(input)?;
    alt((
        |input| cons(pool, input),
        |input| integer(pool, input),
        |input| symbol(pool, input),
    ))(input)
}
