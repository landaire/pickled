// Copyright (c) 2015-2021 Georg Brandl.  Licensed under the Apache License,
// Version 2.0 <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0>
// or the MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at
// your option. This file may not be copied, modified, or distributed except
// according to those terms.

//! Python values, and serialization instances for them.

use num_bigint::BigInt;
use num_traits::{Signed, ToPrimitive};
use std::borrow::Cow;
use std::cell::{Ref, RefCell, RefMut};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::rc::Rc;

pub use crate::value_impls::{from_value, to_value};

use crate::error::{Error, ErrorCode};

#[derive(Debug, Eq, PartialOrd, Ord, Clone)]
pub struct Shared<T>(Rc<RefCell<T>>);

impl<T> Shared<T> {
    pub fn new(value: T) -> Self {
        Shared(Rc::new(RefCell::new(value)))
    }

    pub fn inner<'a>(&'a self) -> Ref<'a, T> {
        self.0.borrow()
    }

    pub fn inner_mut<'a>(&'a self) -> RefMut<'a, T> {
        self.0.borrow_mut()
    }

    pub fn provenance(&self) -> usize {
        Rc::as_ptr(&self.0).expose_provenance()
    }
}

impl<T> From<T> for Shared<T> {
    fn from(value: T) -> Self {
        Shared::new(value)
    }
}

impl<T> From<SharedFrozen<T>> for Shared<T>
where
    T: Clone,
{
    fn from(value: SharedFrozen<T>) -> Self {
        Shared::new(value.into_raw_or_cloned())
    }
}

impl<T> Shared<T>
where
    T: Clone,
{
    pub fn into_raw_or_cloned(self) -> T {
        if Rc::strong_count(&self.0) == 1 {
            if let Some(inner) = Rc::into_inner(self.0) {
                RefCell::into_inner(inner)
            } else {
                panic!("TOCTOU while trying to serialize Shared")
            }
        } else {
            self.0.borrow().clone()
        }
    }
}

impl<T> std::cmp::PartialEq for Shared<T>
where
    T: std::cmp::PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        if Rc::ptr_eq(&self.0, &other.0) {
            return true;
        }

        let this_inner = self.0.borrow();
        let other_inner = other.0.borrow();

        this_inner.eq(&other_inner)
    }
}

#[derive(Debug, Eq, PartialOrd, Ord, Clone)]
pub struct SharedFrozen<T>(Rc<T>);

impl<T> SharedFrozen<T> {
    pub fn new(value: T) -> Self {
        SharedFrozen(Rc::new(value))
    }

    pub fn inner<'a>(&'a self) -> &T {
        self.0.as_ref()
    }

    pub fn provenance(&self) -> usize {
        Rc::as_ptr(&self.0).expose_provenance()
    }
}

impl<T> From<T> for SharedFrozen<T> {
    fn from(value: T) -> Self {
        SharedFrozen::new(value)
    }
}

impl<T> From<Shared<T>> for SharedFrozen<T>
where
    T: Clone,
{
    fn from(value: Shared<T>) -> Self {
        SharedFrozen::new(value.into_raw_or_cloned())
    }
}

impl<T> SharedFrozen<T>
where
    T: Clone,
{
    pub fn into_raw_or_cloned(self) -> T {
        if Rc::strong_count(&self.0) == 1 {
            if let Some(inner) = Rc::into_inner(self.0) {
                inner
            } else {
                panic!("TOCTOU while trying to serialize Shared")
            }
        } else {
            self.inner().clone()
        }
    }
}

impl<T> std::cmp::PartialEq for SharedFrozen<T>
where
    T: std::cmp::PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        if Rc::ptr_eq(&self.0, &other.0) {
            return true;
        }

        let this_inner = self.inner();
        let other_inner = other.inner();

        this_inner.eq(other_inner)
    }
}

/// Represents all primitive builtin Python values that can be restored by
/// unpickling.
///
/// Note on integers: the distinction between the two types (short and long) is
/// very fuzzy in Python, and they can be used interchangeably.  In Python 3,
/// all integers are long integers, so all are pickled as such.  While decoding,
/// we simply put all integers that fit into an i64, and use `BigInt` for the
/// rest.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "variantly", derive(variantly::Variantly))]
pub enum Value {
    /// None
    None,
    /// Boolean
    Bool(bool),
    /// Short integer
    I64(i64),
    /// Long integer (unbounded length)
    Int(BigInt),
    /// Float
    F64(f64),
    /// Bytestring
    Bytes(SharedFrozen<Vec<u8>>),
    /// Unicode string
    String(SharedFrozen<String>),
    /// List
    List(Shared<Vec<Value>>),
    /// Tuple
    Tuple(SharedFrozen<Vec<Value>>),
    /// Set
    Set(Shared<BTreeSet<HashableValue>>),
    /// Frozen (immutable) set
    FrozenSet(SharedFrozen<BTreeSet<HashableValue>>),
    /// Dictionary (map)
    Dict(Shared<BTreeMap<HashableValue, Value>>),
}

/// Represents all primitive builtin Python values that can be contained
/// in a "hashable" context (i.e., as dictionary keys and set elements).
///
/// In Rust, the type is *not* hashable, since we use B-tree maps and sets
/// instead of the hash variants.  To be able to put all Value instances
/// into these B-trees, we implement a consistent ordering between all
/// the possible types (see below).
#[derive(Clone, Debug)]
#[cfg_attr(feature = "variantly", derive(variantly::Variantly))]
pub enum HashableValue {
    /// None
    None,
    /// Boolean
    Bool(bool),
    /// Short integer
    I64(i64),
    /// Long integer
    Int(BigInt),
    /// Float
    F64(f64),
    /// Bytestring
    Bytes(SharedFrozen<Vec<u8>>),
    /// Unicode string
    String(SharedFrozen<String>),
    /// Tuple
    Tuple(SharedFrozen<Vec<HashableValue>>),
    /// Frozen (immutable) set
    FrozenSet(SharedFrozen<BTreeSet<HashableValue>>),
}

fn values_to_raw_hashable(
    values: SharedFrozen<Vec<Value>>,
) -> Result<SharedFrozen<Vec<RawHashableValue>>, Error> {
    Ok(values
        .inner()
        .iter()
        .cloned()
        .map(Value::into_raw_hashable)
        .collect::<Result<Vec<_>, _>>()?
        .into())
}

fn values_to_hashable(
    values: SharedFrozen<Vec<Value>>,
) -> Result<SharedFrozen<Vec<HashableValue>>, Error> {
    Ok(values
        .inner()
        .iter()
        .cloned()
        .map(Value::into_hashable)
        .collect::<Result<Vec<_>, _>>()?
        .into())
}

fn hashable_to_values(values: SharedFrozen<Vec<HashableValue>>) -> SharedFrozen<Vec<Value>> {
    values
        .inner()
        .iter()
        .cloned()
        .map(HashableValue::into_value)
        .collect::<Vec<_>>()
        .into()
}

impl Value {
    /// Convert the value into a hashable version, if possible.  If not, return
    /// a ValueNotHashable error.
    pub fn into_hashable(self) -> Result<HashableValue, Error> {
        match self {
            Value::None => Ok(HashableValue::None),
            Value::Bool(b) => Ok(HashableValue::Bool(b)),
            Value::I64(i) => Ok(HashableValue::I64(i)),
            Value::Int(i) => Ok(HashableValue::Int(i)),
            Value::F64(f) => Ok(HashableValue::F64(f)),
            Value::Bytes(b) => Ok(HashableValue::Bytes(b)),
            Value::String(s) => Ok(HashableValue::String(s)),
            Value::FrozenSet(v) => Ok(HashableValue::FrozenSet(v)),
            Value::Tuple(v) => values_to_hashable(v).map(HashableValue::Tuple),
            _ => Err(Error::Syntax(ErrorCode::ValueNotHashable)),
        }
    }

    pub(crate) fn into_raw_hashable(self) -> Result<RawHashableValue, Error> {
        match self {
            Value::None => Ok(RawHashableValue::None),
            Value::Bool(b) => Ok(RawHashableValue::Bool(b)),
            Value::I64(i) => Ok(RawHashableValue::I64(i)),
            Value::Int(i) => Ok(RawHashableValue::Int(i)),
            Value::F64(f) => Ok(RawHashableValue::F64(f)),
            Value::Bytes(b) => Ok(RawHashableValue::Bytes(b)),
            Value::String(s) => Ok(RawHashableValue::String(s)),
            Value::FrozenSet(v) => {
                let v = v.inner();
                let new = BTreeSet::from_iter(v.iter().cloned().map(|v| {
                    v.into_value()
                        .into_raw_hashable()
                        .expect("failed to round-trip")
                }));

                Ok(RawHashableValue::FrozenSet(SharedFrozen::new(new)))
            }
            Value::Tuple(v) => values_to_raw_hashable(v).map(RawHashableValue::Tuple),
            _ => Err(Error::Syntax(ErrorCode::ValueNotHashable)),
        }
    }
}

impl HashableValue {
    /// Convert the value into its non-hashable version.  This always works.
    pub fn into_value(self) -> Value {
        match self {
            HashableValue::None => Value::None,
            HashableValue::Bool(b) => Value::Bool(b),
            HashableValue::I64(i) => Value::I64(i),
            HashableValue::Int(i) => Value::Int(i),
            HashableValue::F64(f) => Value::F64(f),
            HashableValue::Bytes(b) => Value::Bytes(b),
            HashableValue::String(s) => Value::String(s),
            HashableValue::FrozenSet(v) => Value::FrozenSet(v),
            HashableValue::Tuple(v) => Value::Tuple(hashable_to_values(v)),
        }
    }

    /// Returns a value that's suitable for use as a string key
    pub fn to_string_key(&self) -> Option<Cow<'static, str>> {
        let result = match *self {
            HashableValue::String(ref s) => Cow::Owned(s.inner().to_owned()),
            HashableValue::None => Cow::Borrowed("null"),
            HashableValue::Bool(b) => Cow::Owned(b.to_string()),
            HashableValue::I64(i) => Cow::Owned(i.to_string()),
            HashableValue::Int(ref big_int) => Cow::Owned(big_int.to_string()),
            HashableValue::F64(f) => {
                let mut as_str = f.to_string();
                if !as_str.contains('.') {
                    as_str += ".0";
                }

                Cow::Owned(as_str)
            }
            _ => {
                // All other key types are invalid
                return None;
            }
        };

        Some(result)
    }
}

fn write_elements<'a, I, T>(
    f: &mut fmt::Formatter,
    it: I,
    prefix: &'static str,
    suffix: &'static str,
    len: usize,
    always_comma: bool,
) -> fmt::Result
where
    I: Iterator<Item = &'a T>,
    T: fmt::Display + 'a,
{
    f.write_str(prefix)?;
    for (i, item) in it.enumerate() {
        if i < len - 1 || always_comma {
            write!(f, "{item}, ")?;
        } else {
            write!(f, "{item}")?;
        }
    }
    f.write_str(suffix)
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Value::None => write!(f, "None"),
            Value::Bool(b) => write!(f, "{}", if b { "True" } else { "False" }),
            Value::I64(i) => write!(f, "{i}"),
            Value::Int(ref i) => write!(f, "{i}"),
            Value::F64(v) => write!(f, "{v}"),
            Value::Bytes(ref b) => write!(f, "b{b:?}"),
            Value::String(ref s) => write!(f, "{s:?}"),
            Value::List(ref v) => {
                let v = v.inner();
                write_elements(f, v.iter(), "[", "]", v.len(), false)
            }
            Value::Tuple(ref v) => {
                let v = v.inner();
                write_elements(f, v.iter(), "(", ")", v.len(), v.len() == 1)
            }
            Value::FrozenSet(ref v) => {
                let v = v.inner();
                write_elements(f, v.iter(), "frozenset([", "])", v.len(), false)
            }
            Value::Set(ref v) => {
                let v = v.inner();
                if v.is_empty() {
                    write!(f, "set()")
                } else {
                    write_elements(f, v.iter(), "{", "}", v.len(), false)
                }
            }
            Value::Dict(ref v) => {
                write!(f, "{{")?;
                let v = v.inner();
                for (i, (key, value)) in v.iter().enumerate() {
                    if i < v.len() - 1 {
                        write!(f, "{key}: {value}, ")?;
                    } else {
                        write!(f, "{key}: {value}")?;
                    }
                }
                write!(f, "}}")
            }
        }
    }
}

impl fmt::Display for HashableValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            HashableValue::None => write!(f, "None"),
            HashableValue::Bool(b) => write!(f, "{}", if b { "True" } else { "False" }),
            HashableValue::I64(i) => write!(f, "{i}"),
            HashableValue::Int(ref i) => write!(f, "{i}"),
            HashableValue::F64(v) => write!(f, "{v}"),
            HashableValue::Bytes(ref b) => {
                let b = b.inner();
                write!(f, "b{b:?}")
            }
            HashableValue::String(ref s) => {
                let s = s.inner();
                write!(f, "{s:?}")
            }
            HashableValue::Tuple(ref v) => {
                let v = v.inner();
                write_elements(f, v.iter(), "(", ")", v.len(), v.len() == 1)
            }
            HashableValue::FrozenSet(ref v) => {
                let v = v.inner();
                write_elements(f, v.iter(), "frozenset([", "])", v.len(), false)
            }
        }
    }
}

impl PartialEq for HashableValue {
    fn eq(&self, other: &HashableValue) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for HashableValue {}

impl PartialOrd for HashableValue {
    fn partial_cmp(&self, other: &HashableValue) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Implement a (more or less) consistent ordering for `HashableValue`s
/// so that they can be added to dictionaries and sets.
///
/// Also, like in Python, numeric values with the same value (integral or not)
/// must compare equal.
///
/// For other types, we define an ordering between all types A and B so that all
/// objects of type A are always lesser than objects of type B.  This is done
/// similar to Python 2's ordering of different types.
impl Ord for HashableValue {
    fn cmp(&self, other: &HashableValue) -> Ordering {
        use self::HashableValue::*;
        match *self {
            None => match *other {
                None => Ordering::Equal,
                _ => Ordering::Less,
            },
            Bool(b) => match *other {
                None => Ordering::Greater,
                Bool(b2) => b.cmp(&b2),
                I64(i2) => (b as i64).cmp(&i2),
                Int(ref bi) => BigInt::from(b as i64).cmp(bi),
                F64(f) => float_ord(b as i64 as f64, f),
                _ => Ordering::Less,
            },
            I64(i) => match *other {
                None => Ordering::Greater,
                Bool(b) => i.cmp(&(b as i64)),
                I64(i2) => i.cmp(&i2),
                Int(ref bi) => BigInt::from(i).cmp(bi),
                F64(f) => float_ord(i as f64, f),
                _ => Ordering::Less,
            },
            Int(ref bi) => match *other {
                None => Ordering::Greater,
                Bool(b) => bi.cmp(&BigInt::from(b as i64)),
                I64(i) => bi.cmp(&BigInt::from(i)),
                Int(ref bi2) => bi.cmp(bi2),
                F64(f) => float_bigint_ord(bi, f),
                _ => Ordering::Less,
            },
            F64(f) => match *other {
                None => Ordering::Greater,
                Bool(b) => float_ord(f, b as i64 as f64),
                I64(i) => float_ord(f, i as f64),
                Int(ref bi) => BigInt::from(f as i64).cmp(bi),
                F64(f2) => float_ord(f, f2),
                _ => Ordering::Less,
            },
            Bytes(ref bs) => match *other {
                String(_) | FrozenSet(_) | Tuple(_) => Ordering::Less,
                Bytes(ref bs2) => bs.cmp(bs2),
                _ => Ordering::Greater,
            },
            String(ref s) => match *other {
                FrozenSet(_) | Tuple(_) => Ordering::Less,
                String(ref s2) => s.cmp(s2),
                _ => Ordering::Greater,
            },
            FrozenSet(ref s) => match *other {
                Tuple(_) => Ordering::Less,
                FrozenSet(ref s2) => s.cmp(s2),
                _ => Ordering::Greater,
            },
            Tuple(ref t) => match *other {
                Tuple(ref t2) => t.cmp(t2),
                _ => Ordering::Greater,
            },
        }
    }
}

/// A "reasonable" total ordering for floats.
fn float_ord(f: f64, g: f64) -> Ordering {
    match f.partial_cmp(&g) {
        Some(o) => o,
        None => Ordering::Less,
    }
}

/// A "reasonable" total ordering for floats.
fn total_float_ord(f: f64, g: f64) -> Ordering {
    f.total_cmp(&g)
}

/// Ordering between floats and big integers.
fn float_bigint_ord(bi: &BigInt, g: f64) -> Ordering {
    match bi.to_f64() {
        Some(f) => float_ord(f, g),
        None => {
            if bi.is_positive() {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "variantly", derive(variantly::Variantly))]
pub(crate) enum RawHashableValue {
    /// None
    None,
    /// Boolean
    Bool(bool),
    /// Short integer
    I64(i64),
    /// Long integer
    Int(BigInt),
    /// Float
    F64(f64),
    /// Bytestring
    Bytes(SharedFrozen<Vec<u8>>),
    /// Unicode string
    String(SharedFrozen<String>),
    /// Tuple
    Tuple(SharedFrozen<Vec<RawHashableValue>>),
    /// Frozen (immutable) set
    FrozenSet(SharedFrozen<BTreeSet<RawHashableValue>>),
}

impl std::cmp::Eq for RawHashableValue {}

impl std::cmp::PartialOrd for RawHashableValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for RawHashableValue {
    #[inline]
    fn cmp(&self, other: &RawHashableValue) -> ::core::cmp::Ordering {
        let __self_discr = match self {
            RawHashableValue::None => 0,
            RawHashableValue::Bool(_) => 1,
            RawHashableValue::I64(_) => 2,
            RawHashableValue::Int(_) => 3,
            RawHashableValue::F64(_) => 4,
            RawHashableValue::Bytes(_) => 5,
            RawHashableValue::String(_) => 6,
            RawHashableValue::Tuple(_) => 7,
            RawHashableValue::FrozenSet(_) => 8,
        };
        let __arg1_discr = match other {
            RawHashableValue::None => 0,
            RawHashableValue::Bool(_) => 1,
            RawHashableValue::I64(_) => 2,
            RawHashableValue::Int(_) => 3,
            RawHashableValue::F64(_) => 4,
            RawHashableValue::Bytes(_) => 5,
            RawHashableValue::String(_) => 6,
            RawHashableValue::Tuple(_) => 7,
            RawHashableValue::FrozenSet(_) => 8,
        };

        match ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr) {
            ::core::cmp::Ordering::Equal => match (self, other) {
                (RawHashableValue::Bool(__self_0), RawHashableValue::Bool(__arg1_0)) => {
                    ::core::cmp::Ord::cmp(__self_0, __arg1_0)
                }
                (RawHashableValue::I64(__self_0), RawHashableValue::I64(__arg1_0)) => {
                    ::core::cmp::Ord::cmp(__self_0, __arg1_0)
                }
                (RawHashableValue::Int(__self_0), RawHashableValue::Int(__arg1_0)) => {
                    ::core::cmp::Ord::cmp(__self_0, __arg1_0)
                }
                (RawHashableValue::Bytes(__self_0), RawHashableValue::Bytes(__arg1_0)) => {
                    ::core::cmp::Ord::cmp(__self_0, __arg1_0)
                }
                (RawHashableValue::String(__self_0), RawHashableValue::String(__arg1_0)) => {
                    ::core::cmp::Ord::cmp(__self_0, __arg1_0)
                }
                (RawHashableValue::Tuple(__self_0), RawHashableValue::Tuple(__arg1_0)) => {
                    ::core::cmp::Ord::cmp(__self_0, __arg1_0)
                }
                (RawHashableValue::FrozenSet(__self_0), RawHashableValue::FrozenSet(__arg1_0)) => {
                    ::core::cmp::Ord::cmp(__self_0, __arg1_0)
                }
                (RawHashableValue::F64(__self_0), RawHashableValue::F64(__self_1)) => {
                    total_float_ord(*__self_0, *__self_1)
                }
                _ => ::core::cmp::Ordering::Equal,
            },
            cmp => cmp,
        }
    }
}
