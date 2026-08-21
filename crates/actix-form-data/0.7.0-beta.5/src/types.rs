/*
 * This file is part of Actix Form Data.
 *
 * Copyright © 2020 Riley Trautman
 *
 * Actix Form Data is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * Actix Form Data is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with Actix Form Data.  If not, see <http://www.gnu.org/licenses/>.
 */

use crate::Error;
use actix_web::web::Bytes;
use futures_core::Stream;
use mime::Mime;
use std::{
    collections::{HashMap, VecDeque},
    fmt,
    future::Future,
    pin::Pin,
};
use tracing::trace;

#[derive(Debug)]
pub struct FileMeta<T> {
    pub filename: String,
    pub content_type: Option<Mime>,
    pub result: T,
}

/// The result of a succesfull parse through a given multipart stream.
///
/// This type represents all possible variations in structure of a Multipart Form.
///
/// # Example usage
///
/// ```rust
/// # use actix_form_data::Value;
/// # use std::collections::HashMap;
/// # let mut hm = HashMap::new();
/// # hm.insert("field-name".to_owned(), Value::Int(32));
/// # let value = Value::<()>::Map(hm);
/// if let Value::Map(mut hashmap) = value {
///     if let Some(Value::Int(integer)) = hashmap.remove("field-name") {
///         println!("{}", integer);
///     }
/// }
/// ```
#[derive(Debug)]
pub enum Value<T> {
    Map(HashMap<String, Value<T>>),
    Array(Vec<Value<T>>),
    File(FileMeta<T>),
    Bytes(Bytes),
    Text(String),
    Int(i64),
    Float(f64),
}

impl<T> Value<T> {
    pub(crate) fn merge(&mut self, rhs: Self) {
        match self {
            Value::Map(ref mut hm) => {
                if let Value::Map(other) = rhs {
                    other.into_iter().fold(hm, |hm, (key, value)| {
                        if let Some(v) = hm.get_mut(&key) {
                            v.merge(value);
                        } else {
                            hm.insert(key.to_owned(), value);
                        }

                        hm
                    });
                }
            }
            Value::Array(ref mut v) => {
                if let Value::Array(other) = rhs {
                    v.extend(other);
                }
            }
            _ => (),
        }
    }

    pub fn map(self) -> Option<HashMap<String, Value<T>>> {
        match self {
            Value::Map(map) => Some(map),
            _ => None,
        }
    }

    pub fn array(self) -> Option<Vec<Value<T>>> {
        match self {
            Value::Array(vec) => Some(vec),
            _ => None,
        }
    }

    pub fn file(self) -> Option<FileMeta<T>> {
        match self {
            Value::File(file_meta) => Some(file_meta),
            _ => None,
        }
    }

    pub fn bytes(self) -> Option<Bytes> {
        match self {
            Value::Bytes(bytes) => Some(bytes),
            _ => None,
        }
    }

    pub fn text(self) -> Option<String> {
        match self {
            Value::Text(text) => Some(text),
            _ => None,
        }
    }

    pub fn int(self) -> Option<i64> {
        match self {
            Value::Int(int) => Some(int),
            _ => None,
        }
    }

    pub fn float(self) -> Option<f64> {
        match self {
            Value::Float(float) => Some(float),
            _ => None,
        }
    }
}

impl<T> From<MultipartContent<T>> for Value<T> {
    fn from(mc: MultipartContent<T>) -> Self {
        match mc {
            MultipartContent::File(file_meta) => Value::File(file_meta),
            MultipartContent::Bytes(bytes) => Value::Bytes(bytes),
            MultipartContent::Text(string) => Value::Text(string),
            MultipartContent::Int(i) => Value::Int(i),
            MultipartContent::Float(f) => Value::Float(f),
        }
    }
}

pub type FileFn<T, E> = Box<
    dyn Fn(
        String,
        Option<Mime>,
        Pin<Box<dyn Stream<Item = Result<Bytes, Error>>>>,
    ) -> Pin<Box<dyn Future<Output = Result<T, E>>>>,
>;

/// The field type represents a field in the form-data that is allowed to be parsed.
pub enum Field<T, E> {
    Array(Array<T, E>),
    Map(Map<T, E>),
    File(FileFn<T, E>),
    Bytes,
    Int,
    Float,
    Text,
}

impl<T, E> fmt::Debug for Field<T, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Field::Array(ref arr) => f.debug_tuple("Array").field(arr).finish(),
            Field::Map(ref map) => f.debug_tuple("Map").field(map).finish(),
            Field::File(_) => write!(f, "File"),
            Field::Bytes => write!(f, "Bytes"),
            Field::Int => write!(f, "Int"),
            Field::Float => write!(f, "Float"),
            Field::Text => write!(f, "Text"),
        }
    }
}

impl<T, E> Field<T, E> {
    /// Add a File field with a name generator.
    ///
    /// The name generator will be called for each file matching this field's key. Keep in mind
    /// that each key/file pair will have it's own name-generator, so sharing a name-generator
    /// between fields is up to the user.
    ///
    /// # Example
    /// ```rust
    /// # use actix_form_data::{Error, Form, Field};
    /// # use tokio::sync::mpsc::channel;
    /// # use futures_util::stream::StreamExt;
    /// #
    /// let (tx, rx) = channel(1);
    /// let form = Form::new().field("file-field", Field::file(move |_, _, mut stream| {
    ///     let mut tx = tx.clone();
    ///     async move {
    ///         while let Some(res) = stream.next().await {
    ///             if let Ok(bytes) = res {
    ///                 if let Err(_) = tx.send(bytes).await {
    ///                     break;
    ///                 }
    ///             }
    ///         }
    ///         Ok(()) as Result<_, Error>
    ///     }
    /// }));
    /// ```
    pub fn file<F, Fut>(f: F) -> Self
    where
        F: Fn(String, Option<Mime>, Pin<Box<dyn Stream<Item = Result<Bytes, Error>>>>) -> Fut
            + Clone
            + 'static,
        Fut: Future<Output = Result<T, E>> + 'static,
        E: 'static,
    {
        Field::File(Box::new(move |filename, mime, stream| {
            let f = f.clone();
            Box::pin((f)(filename, mime, stream))
        }))
    }

    /// Add a Bytes field to a form
    ///
    /// # Example
    /// ```rust
    /// # use actix_form_data::{Error, Form, Field};
    /// let form = Form::<(), Error>::new().field("text-field", Field::bytes());
    pub const fn bytes() -> Self {
        Field::Bytes
    }

    /// Add a Text field to a form
    ///
    /// # Example
    /// ```rust
    /// # use actix_form_data::{Error, Form, Field};
    /// let form = Form::<(), Error>::new().field("text-field", Field::text());
    pub const fn text() -> Self {
        Field::Text
    }

    /// Add an Int field to a form
    ///
    /// # Example
    /// ```rust
    /// # use actix_form_data::{Error, Form, Field};
    /// let form = Form::<(), Error>::new().field("int-field", Field::int());
    /// ```
    pub const fn int() -> Self {
        Field::Int
    }

    /// Add a Float field to a form
    ///
    /// # Example
    /// ```rust
    /// # use actix_form_data::{Error, Form, Field};
    /// let form = Form::<(), Error>::new().field("float-field", Field::float());
    /// ```
    pub const fn float() -> Self {
        Field::Float
    }

    /// Add an Array to a form
    ///
    /// # Example
    /// ```rust
    /// # use actix_form_data::{Error, Form, Field};
    /// # fn main() {
    /// let form = Form::<(), Error>::new()
    ///     .field(
    ///         "array-field",
    ///         Field::array(Field::text())
    ///     );
    /// # }
    /// ```
    pub fn array(field: Field<T, E>) -> Self {
        Field::Array(Array::new(field))
    }

    /// Add a Map to a form
    ///
    /// # Example
    /// ```rust
    /// # use actix_form_data::{Error, Form, Field};
    /// # fn main() {
    /// let form = Form::<(), Error>::new()
    ///     .field(
    ///         "map-field",
    ///         Field::map()
    ///             .field("sub-field", Field::text())
    ///             .field("sub-field-two", Field::text())
    ///             .finalize()
    ///     );
    /// # }
    /// ```
    pub const fn map() -> Map<T, E> {
        Map::new()
    }

    fn valid_field<'a>(&'a self, name: VecDeque<&NamePart>) -> Option<FieldTerminator<'a, T, E>> {
        trace!("Checking {:?} and {:?}", self, name);
        match *self {
            Field::Array(ref arr) => arr.valid_field(name),
            Field::Map(ref map) => map.valid_field(name),
            Field::File(ref file_fn) => {
                if name.is_empty() {
                    Some(FieldTerminator::File(file_fn))
                } else {
                    None
                }
            }
            Field::Bytes => {
                if name.is_empty() {
                    Some(FieldTerminator::Bytes)
                } else {
                    None
                }
            }
            Field::Int => {
                if name.is_empty() {
                    Some(FieldTerminator::Int)
                } else {
                    None
                }
            }
            Field::Float => {
                if name.is_empty() {
                    Some(FieldTerminator::Float)
                } else {
                    None
                }
            }
            Field::Text => {
                if name.is_empty() {
                    Some(FieldTerminator::Text)
                } else {
                    None
                }
            }
        }
    }
}

/// A definition of an array of type `Field` to be parsed from form data.
///
/// The `Array` type should only be constructed in the context of a Form. See the `Form`
/// documentation for more information.
pub struct Array<T, E> {
    inner: Box<Field<T, E>>,
}

impl<T, E> fmt::Debug for Array<T, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Array").field("inner", &self.inner).finish()
    }
}

impl<T, E> Array<T, E> {
    fn new(field: Field<T, E>) -> Self {
        Array {
            inner: Box::new(field),
        }
    }

    fn valid_field<'a>(
        &'a self,
        mut name: VecDeque<&NamePart>,
    ) -> Option<FieldTerminator<'a, T, E>> {
        trace!("Checking {:?} and {:?}", self, name);
        match name.pop_front() {
            Some(NamePart::Array) => self.inner.valid_field(name),
            _ => None,
        }
    }
}

/// A definition of key-value pairs to be parsed from form data.
pub struct Map<T, E> {
    inner: Vec<(String, Field<T, E>)>,
}

impl<T, E> fmt::Debug for Map<T, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Map").field("inner", &self.inner).finish()
    }
}

impl<T, E> Map<T, E> {
    const fn new() -> Self {
        Map { inner: Vec::new() }
    }

    /// Add a `Field` to a map
    /// # Example
    /// ```rust
    /// # use actix_form_data::{Error, Field};
    /// #
    /// Field::<(), Error>::map()
    ///     .field("sub-field", Field::text())
    ///     .field("sub-field-two", Field::text())
    ///     .finalize();
    /// ```
    pub fn field(mut self, key: &str, value: Field<T, E>) -> Self {
        self.inner.push((key.to_owned(), value));

        self
    }

    /// Finalize the map into a `Field`, so it can be added to a Form
    /// ```rust
    /// # use actix_form_data::{Error, Field};
    /// #
    /// Field::<(), Error>::map()
    ///     .field("sub-field", Field::text())
    ///     .field("sub-field-two", Field::text())
    ///     .finalize();
    /// ```
    pub const fn finalize(self) -> Field<T, E> {
        Field::Map(self)
    }

    fn valid_field<'a>(
        &'a self,
        mut name: VecDeque<&NamePart>,
    ) -> Option<FieldTerminator<'a, T, E>> {
        trace!("Checking {:?} and {:?}", self, name);
        match name.pop_front() {
            Some(NamePart::Map(name_part)) => self
                .inner
                .iter()
                .find(|(ref item, _)| *item == *name_part)
                .and_then(|(_, ref field)| field.valid_field(name)),
            _ => None,
        }
    }
}

/// A structure that defines the fields expected in form data
///
/// # Example
/// ```rust
/// # use actix_form_data::{Error, Form, Field};
/// let form = Form::<(), Error>::new()
///     .field("field-name", Field::text())
///     .field("second-field", Field::int())
///     .field("third-field", Field::float())
///     .field("fifth-field", Field::file(|_, _, _| async move {
///         Ok(())
///     }))
///     .field(
///         "map-field",
///         Field::map()
///             .field("sub-field", Field::text())
///             .field("sub-field-two", Field::text())
///             .finalize()
///     )
///     .field(
///         "array-field",
///         Field::array(Field::text())
///     );
/// ```
pub struct Form<T, E> {
    pub(crate) max_fields: u32,
    pub(crate) max_field_size: usize,
    pub(crate) max_files: u32,
    pub(crate) max_file_size: usize,
    pub(crate) transform_error: Option<Box<dyn Fn(Error) -> actix_web::Error + Sync>>,
    inner: Map<T, E>,
}

impl<T, E> Default for Form<T, E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, E> Form<T, E> {
    /// Create a new form
    ///
    /// If you wish to provide your own executor, use the `with_executor` method.
    ///
    /// Default values are as follows
    /// - max_fields: 100
    /// - max_field_size: 10_000 bytes
    /// - max_files: 20
    /// - max_files_size: 10_000_000 bytes
    pub const fn new() -> Self {
        Form {
            max_fields: 100,
            max_field_size: 10_000,
            max_files: 20,
            max_file_size: 10_000_000,
            transform_error: None,
            inner: Map::new(),
        }
    }

    /// Set the Transform Error method to convert Error types into actix_web::Error by hand
    pub fn transform_error(
        mut self,
        f: impl Fn(Error) -> actix_web::Error + Sync + 'static,
    ) -> Self {
        self.transform_error = Some(Box::new(f));
        self
    }

    /// Set the maximum number of fields allowed in the upload
    ///
    /// The upload will error if too many fields are provided.
    pub const fn max_fields(mut self, max: u32) -> Self {
        self.max_fields = max;

        self
    }

    /// Set the maximum size of a field (in bytes)
    ///
    /// The upload will error if a provided field is too large.
    pub const fn max_field_size(mut self, max: usize) -> Self {
        self.max_field_size = max;

        self
    }

    /// Set the maximum number of files allowed in the upload
    ///
    /// THe upload will error if too many files are provided.
    pub const fn max_files(mut self, max: u32) -> Self {
        self.max_files = max;

        self
    }

    /// Set the maximum size for files (in bytes)
    ///
    /// The upload will error if a provided file is too large.
    pub const fn max_file_size(mut self, max: usize) -> Self {
        self.max_file_size = max;

        self
    }

    pub fn field(mut self, name: &str, field: Field<T, E>) -> Self {
        self.inner = self.inner.field(name, field);

        self
    }

    pub(crate) fn valid_field<'a>(
        &'a self,
        name: VecDeque<&NamePart>,
    ) -> Option<FieldTerminator<'a, T, E>> {
        self.inner.valid_field(name)
    }
}

impl<T, E> fmt::Debug for Form<T, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Form").field("inner", &self.inner).finish()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ContentDisposition {
    pub name: Option<String>,
    pub filename: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum NamePart {
    Map(String),
    Array,
}

impl NamePart {
    pub const fn is_map(&self) -> bool {
        matches!(self, NamePart::Map(_))
    }
}

#[derive(Clone)]
pub(crate) enum FieldTerminator<'a, T, E> {
    File(&'a FileFn<T, E>),
    Bytes,
    Int,
    Float,
    Text,
}

impl<'a, T, E> fmt::Debug for FieldTerminator<'a, T, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FieldTerminator::File(_) => write!(f, "File"),
            FieldTerminator::Bytes => write!(f, "Bytes"),
            FieldTerminator::Int => write!(f, "Int"),
            FieldTerminator::Float => write!(f, "Float"),
            FieldTerminator::Text => write!(f, "Text"),
        }
    }
}

pub(crate) type MultipartHash<T> = (Vec<NamePart>, MultipartContent<T>);
pub(crate) type MultipartForm<T> = Vec<MultipartHash<T>>;

#[derive(Debug)]
pub(crate) enum MultipartContent<T> {
    File(FileMeta<T>),
    Bytes(Bytes),
    Text(String),
    Int(i64),
    Float(f64),
}
