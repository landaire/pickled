// Copyright (c) 2015-2021 Georg Brandl.  Licensed under the Apache License,
// Version 2.0 <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0>
// or the MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at
// your option. This file may not be copied, modified, or distributed except
// according to those terms.

//! QuickCheck Arbitrary instance for Value, and associated helpers.

use crate::value::{Shared, SharedFrozen};
use crate::{HashableValue, Value};
use num_bigint::BigInt;
use quickcheck::{Arbitrary, Gen, empty_shrinker};
use std::{i64, ops::Range};

const MAX_DEPTH: u32 = 1;

fn gen_value(g: &mut Gen, depth: u32) -> Value {
    let upper = if depth > 0 { 12 } else { 7 };
    match gen_range(0..upper, g) {
        // leaves
        0 => Value::None,
        1 => Value::Bool(Arbitrary::arbitrary(g)),
        2 => Value::I64(Arbitrary::arbitrary(g)),
        3 => Value::Int(gen_bigint(g)),
        4 => Value::F64(Arbitrary::arbitrary(g)),
        5 => Value::Bytes(SharedFrozen::new(Arbitrary::arbitrary(g))),
        6 => Value::String(SharedFrozen::new(Arbitrary::arbitrary(g))),
        // recursive variants
        7 => Value::List(Shared::new(gen_vec(g, depth - 1))),
        8 => Value::Tuple(SharedFrozen::new(gen_vec(g, depth - 1))),
        9 => Value::Set(Shared::new(gen_hvec(g, depth - 1).into_iter().collect())),
        10 => Value::FrozenSet(SharedFrozen::new(
            gen_hvec(g, depth - 1).into_iter().collect(),
        )),
        11 => {
            let kvec = gen_hvec(g, depth - 1);
            let vvec = gen_vec(g, depth - 1);
            Value::Dict(Shared::new(kvec.into_iter().zip(vvec).collect()))
        }
        _ => unreachable!(),
    }
}

fn gen_bigint(g: &mut Gen) -> BigInt {
    // We have to construct a value outside of i64 range, since other values
    // are unpickled as i64s instead of big ints.
    let offset = BigInt::from(2)
        * BigInt::from(if bool::arbitrary(g) {
            i64::MIN
        } else {
            i64::MAX
        });
    offset + BigInt::from(i64::arbitrary(g))
}

fn gen_vec(g: &mut Gen, depth: u32) -> Vec<Value> {
    let size = {
        let s = g.size();
        gen_range(0..s, g)
    };
    (0..size).map(|_| gen_value(g, depth)).collect()
}

fn gen_hvalue(g: &mut Gen, depth: u32) -> HashableValue {
    let upper = if depth > 0 { 9 } else { 7 };
    match gen_range(0..upper, g) {
        // leaves
        0 => HashableValue::None,
        1 => HashableValue::Bool(Arbitrary::arbitrary(g)),
        2 => HashableValue::I64(Arbitrary::arbitrary(g)),
        3 => {
            // We have to construct a value outside of i64 range.
            let val: i64 = Arbitrary::arbitrary(g);
            let max = BigInt::from(i64::MAX);
            HashableValue::Int(BigInt::from(val) + BigInt::from(2) * max)
        }
        4 => HashableValue::F64(Arbitrary::arbitrary(g)),
        5 => HashableValue::Bytes(SharedFrozen::new(Arbitrary::arbitrary(g))),
        6 => HashableValue::String(SharedFrozen::new(Arbitrary::arbitrary(g))),
        // recursive variants
        7 => HashableValue::Tuple(SharedFrozen::new(gen_hvec(g, depth - 1))),
        8 => HashableValue::FrozenSet(SharedFrozen::new(
            gen_hvec(g, depth - 1).into_iter().collect(),
        )),
        _ => unreachable!(),
    }
}

fn gen_hvec(g: &mut Gen, depth: u32) -> Vec<HashableValue> {
    let size = {
        let s = g.size();
        gen_range(0..s, g)
    };
    (0..size).map(|_| gen_hvalue(g, depth)).collect()
}

fn gen_range(r: Range<usize>, g: &mut Gen) -> usize {
    let possibilities = r.into_iter().collect::<Vec<_>>();
    g.choose(possibilities.as_slice()).unwrap().clone()
}

impl Arbitrary for Value {
    fn arbitrary(g: &mut Gen) -> Value {
        gen_value(g, MAX_DEPTH)
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Value>> {
        match *self {
            Value::None => empty_shrinker(),
            Value::Bool(v) => Box::new(Arbitrary::shrink(&v).map(Value::Bool)),
            Value::I64(v) => Box::new(Arbitrary::shrink(&v).map(Value::I64)),
            Value::Int(_) => empty_shrinker(),
            Value::F64(v) => Box::new(Arbitrary::shrink(&v).map(Value::F64)),
            Value::Bytes(ref v) => {
                Box::new(Arbitrary::shrink(&*v.inner()).map(|x| Value::Bytes(SharedFrozen::new(x))))
            }
            Value::String(ref v) => Box::new(
                Arbitrary::shrink(&*v.inner()).map(|x| Value::String(SharedFrozen::new(x))),
            ),
            Value::List(ref v) => {
                Box::new(Arbitrary::shrink(&*v.inner()).map(|x| Value::List(Shared::new(x))))
            }
            Value::Tuple(ref v) => {
                Box::new(Arbitrary::shrink(&*v.inner()).map(|x| Value::Tuple(SharedFrozen::new(x))))
            }
            Value::Set(ref v) => {
                Box::new(Arbitrary::shrink(&*v.inner()).map(|x| Value::Set(Shared::new(x))))
            }
            Value::FrozenSet(ref v) => Box::new(
                Arbitrary::shrink(&*v.inner()).map(|x| Value::FrozenSet(SharedFrozen::new(x))),
            ),
            Value::Dict(ref v) => {
                Box::new(Arbitrary::shrink(&*v.inner()).map(|x| Value::Dict(Shared::new(x))))
            }
        }
    }
}

impl Arbitrary for HashableValue {
    fn arbitrary(g: &mut Gen) -> HashableValue {
        gen_hvalue(g, MAX_DEPTH)
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = HashableValue>> {
        match *self {
            HashableValue::None => empty_shrinker(),
            HashableValue::Bool(v) => Box::new(Arbitrary::shrink(&v).map(HashableValue::Bool)),
            HashableValue::I64(v) => Box::new(Arbitrary::shrink(&v).map(HashableValue::I64)),
            HashableValue::Int(_) => empty_shrinker(),
            HashableValue::F64(v) => Box::new(Arbitrary::shrink(&v).map(HashableValue::F64)),
            HashableValue::Bytes(ref v) => Box::new(
                Arbitrary::shrink(&*v.inner()).map(|x| HashableValue::Bytes(SharedFrozen::new(x))),
            ),
            HashableValue::String(ref v) => Box::new(
                Arbitrary::shrink(&*v.inner()).map(|x| HashableValue::String(SharedFrozen::new(x))),
            ),
            HashableValue::Tuple(ref v) => Box::new(
                Arbitrary::shrink(&*v.inner()).map(|x| HashableValue::Tuple(SharedFrozen::new(x))),
            ),
            HashableValue::FrozenSet(ref v) => Box::new(
                Arbitrary::shrink(&*v.inner())
                    .map(|x| HashableValue::FrozenSet(SharedFrozen::new(x))),
            ),
        }
    }
}
