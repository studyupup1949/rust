use crate::FontError;
use std::fmt;
use tiny_skia::{Path, PathBuilder, Point};

pub mod type1;
pub mod type2;

#[derive(Copy, Clone)]
pub enum Value {
    Int(i32),
    Float(f32),
}
impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(i) => i.fmt(f),
            Value::Float(x) => x.fmt(f),
        }
    }
}

impl Into<f32> for Value {
    #[inline]
    fn into(self) -> f32 {
        self.to_float()
    }
}
impl From<i16> for Value {
    #[inline]
    fn from(v: i16) -> Value {
        Value::Int(v as i32)
    }
}
impl From<i32> for Value {
    #[inline]
    fn from(v: i32) -> Value {
        Value::Int(v)
    }
}
impl From<f32> for Value {
    #[inline]
    fn from(v: f32) -> Value {
        Value::Float(v)
    }
}
impl Value {
    #[inline]
    pub fn to_int(self) -> Result<i32, FontError> {
        match self {
            Value::Int(i) => Ok(i),
            Value::Float(_) => Err(FontError::TypeError("tried to cast a float to int")),
        }
    }
    #[inline]
    pub fn to_float(self) -> f32 {
        match self {
            Value::Int(i) => i as f32,
            Value::Float(f) => f,
        }
    }
}

#[inline]
pub fn v(x: impl Into<f32>, y: impl Into<f32>) -> Point {
    Point::from_xy(x.into(), y.into())
}

pub trait TryIndex {
    fn try_index(&self, idx: usize) -> Option<&[u8]>;
}
impl TryIndex for () {
    #[inline]
    fn try_index(&self, _idx: usize) -> Option<&[u8]> {
        None
    }
}
impl TryIndex for Vec<Option<Vec<u8>>> {
    #[inline]
    fn try_index(&self, idx: usize) -> Option<&[u8]> {
        match self.get(idx) {
            Some(Some(v)) => Some(&**v),
            _ => None,
        }
    }
}
impl TryIndex for Vec<Vec<u8>> {
    #[inline]
    fn try_index(&self, idx: usize) -> Option<&[u8]> {
        self.get(idx).map(|v| &**v)
    }
}
impl<'a> TryIndex for Vec<&'a [u8]> {
    #[inline]
    fn try_index(&self, idx: usize) -> Option<&[u8]> {
        self.get(idx).map(|v| *v)
    }
}
impl<'a> TryIndex for &'a [&'a [u8]] {
    #[inline]
    fn try_index(&self, idx: usize) -> Option<&[u8]> {
        self.get(idx).map(|v| *v)
    }
}

pub struct Context<T = (), U = ()> {
    pub subr_bias: i32,
    pub subrs: T,
    pub global_subrs: U,
    pub global_subr_bias: i32,
}

impl<T, U> Context<T, U>
where
    T: TryIndex,
    U: TryIndex,
{
    #[inline]
    pub fn subr(&self, idx: i32) -> Result<&[u8], FontError> {
        match self.subrs.try_index((idx + self.subr_bias) as usize) {
            Some(sub) => Ok(sub),
            None => error!("requested subroutine {} not found", idx),
        }
    }
    #[inline]
    pub fn global_subr(&self, idx: i32) -> Result<&[u8], FontError> {
        match self
            .global_subrs
            .try_index((idx + self.global_subr_bias) as usize)
        {
            Some(sub) => Ok(sub),
            None => error!("requested global subroutine {} not found", idx),
        }
    }
}

pub struct State {
    pub stack: Vec<Value>,
    pub outline: PathBuilder,
    pub current: Point,
    pub lsb: Option<f32>,
    pub char_width: Option<f32>,
    pub done: bool,
    pub stem_hints: u32,
    pub delta_width: Option<f32>,
    pub first_stack_clearing_operator: bool,
    pub flex_sequence: Option<Vec<Point>>,
}

impl State {
    #[inline]
    pub fn new() -> State {
        State {
            stack: Vec::new(),
            outline: PathBuilder::new(),
            current: Point::default(),
            lsb: None,
            char_width: None,
            done: false,
            stem_hints: 0,
            delta_width: None,
            first_stack_clearing_operator: true,
            flex_sequence: None,
        }
    }
    #[inline]
    pub fn clear(&mut self) {
        self.stack.clear();
        self.outline.clear();
        self.current = Point::default();
        self.lsb = None;
        self.char_width = None;
        self.done = false;
        self.stem_hints = 0;
        self.delta_width = None;
        self.first_stack_clearing_operator = true;
        self.flex_sequence = None;
    }

    #[inline]
    pub fn flush(&mut self) {
        if !self.outline.is_empty() {
            self.outline.close();
        }
    }
    #[inline]
    pub fn take_path(&mut self) -> Option<Path> {
        self.flush();
        let outline = self.outline.clone();
        outline.finish()
    }
    #[inline]
    pub fn push(&mut self, v: impl Into<Value>) {
        self.stack.push(v.into());
    }
    #[inline]
    pub fn pop(&mut self) -> Result<Value, FontError> {
        Ok(expect!(self.stack.pop(), "no value on the stack"))
    }
    #[inline]
    pub fn pop_tuple<const N: usize>(&mut self) -> Result<[Value; N], FontError> {
        if self.stack.len() < N {
            expect!(None, "not enough data on the stack");
        }
        let mut tuple = [Value::Int(0); N];
        for (x, s) in tuple
            .iter_mut()
            .zip(self.stack.drain(self.stack.len() - N..))
        {
            *x = s;
        }
        Ok(tuple)
    }
    /// get stack[0 .. T::N] as a tuple
    /// does not modify the stack
    #[inline]
    pub fn args<const N: usize>(&mut self) -> Result<[Value; N], FontError> {
        if self.stack.len() < N {
            expect!(None, "not enough data on the stack");
        }
        let mut tuple = [Value::Int(0); N];
        for (x, s) in tuple
            .iter_mut()
            .zip(self.stack[self.stack.len() - N..].iter())
        {
            *x = *s;
        }
        Ok(tuple)
    }
}
