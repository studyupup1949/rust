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
use futures::Stream;
use mime::Mime;
use std::{
    collections::{HashMap, VecDeque},
    fmt,
    future::Future,
    path::PathBuf,
    pin::Pin,
    sync::Arc,
};
use tracing::trace;

#[derive(Debug)]
pub struct FileMeta {
    pub filename: String,
    pub content_type: Mime,
    pub saved_as: Option<PathBuf>,
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
/// # let value = Value::Map(hm);
/// match value {
///     Value::Map(mut hashmap) => {
///         match hashmap.remove("field-name") {
///             Some(value) => match value {
///                 Value::Int(integer) => println!("{}", integer),
///                 _ => (),
///             }
///             None => (),
///         }
///     }
///     _ => (),
/// }
/// ```
#[derive(Debug)]
pub enum Value {
    Map(HashMap<String, Value>),
    Array(Vec<Value>),
    File(FileMeta),
    Bytes(Bytes),
    Text(String),
    Int(i64),
    Float(f64),
}

impl Value {
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

    pub fn map(self) -> Option<HashMap<String, Value>> {
        match self {
            Value::Map(map) => Some(map),
            _ => None,
        }
    }

    pub fn array(self) -> Option<Vec<Value>> {
        match self {
            Value::Array(vec) => Some(vec),
            _ => None,
        }
    }

    pub fn file(self) -> Option<FileMeta> {
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

impl From<MultipartContent> for Value {
    fn from(mc: MultipartContent) -> Self {
        match mc {
            MultipartContent::File(file_meta) => Value::File(file_meta),
            MultipartContent::Bytes(bytes) => Value::Bytes(bytes),
            MultipartContent::Text(string) => Value::Text(string),
            MultipartContent::Int(i) => Value::Int(i),
            MultipartContent::Float(f) => Value::Float(f),
        }
    }
}

pub type FileFn = Arc<
    dyn Fn(
            String,
            Mime,
            Pin<Box<dyn Stream<Item = Result<Bytes, Error>>>>,
        ) -> Pin<Box<dyn Future<Output = Result<Option<PathBuf>, actix_web::Error>>>>
        + Send
        + Sync,
>;

/// The field type represents a field in the form-data that is allowed to be parsed.
#[derive(Clone)]
pub enum Field {
    Array(Array),
    Map(Map),
    File(FileFn),
    Bytes,
    Int,
    Float,
    Text,
}

impl fmt::Debug for Field {
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

impl Field {
    /// Add a File field with a name generator.
    ///
    /// The name generator will be called for each file matching this field's key. Keep in mind
    /// that each key/file pair will have it's own name-generator, so sharing a name-generator
    /// between fields is up to the user.
    ///
    /// # Example
    /// ```rust
    /// # use actix_form_data::{Form, Field};
    /// # use tokio::sync::mpsc::channel;
    /// # use futures::stream::StreamExt;
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
    ///         Ok(None) as Result<_, std::convert::Infallible>
    ///     }
    /// }));
    /// ```
    pub fn file<F, Fut, E>(f: F) -> Self
    where
        F: Fn(String, Mime, Pin<Box<dyn Stream<Item = Result<Bytes, Error>>>>) -> Fut
            + Send
            + Sync
            + Clone
            + 'static,
        Fut: Future<Output = Result<Option<PathBuf>, E>> + 'static,
        E: Into<actix_web::Error> + 'static,
    {
        Field::File(Arc::new(move |filename, mime, stream| {
            let f = f.clone();
            Box::pin(async move { (f)(filename, mime, stream).await.map_err(|e| e.into()) })
        }))
    }

    /// Add a Bytes field to a form
    ///
    /// # Example
    /// ```rust
    /// # use actix_form_data::{Form, Field};
    /// let form = Form::new().field("text-field", Field::bytes());
    pub fn bytes() -> Self {
        Field::Bytes
    }

    /// Add a Text field to a form
    ///
    /// # Example
    /// ```rust
    /// # use actix_form_data::{Form, Field};
    /// let form = Form::new().field("text-field", Field::text());
    pub fn text() -> Self {
        Field::Text
    }

    /// Add an Int field to a form
    ///
    /// # Example
    /// ```rust
    /// # use actix_form_data::{Form, Field};
    /// let form = Form::new().field("int-field", Field::int());
    /// ```
    pub fn int() -> Self {
        Field::Int
    }

    /// Add a Float field to a form
    ///
    /// # Example
    /// ```rust
    /// # use actix_form_data::{Form, Field};
    /// let form = Form::new().field("float-field", Field::float());
    /// ```
    pub fn float() -> Self {
        Field::Float
    }

    /// Add an Array to a form
    ///
    /// # Example
    /// ```rust
    /// # use actix_form_data::{Form, Field};
    /// # fn main() {
    /// let form = Form::new()
    ///     .field(
    ///         "array-field",
    ///         Field::array(Field::text())
    ///     );
    /// # }
    /// ```
    pub fn array(field: Field) -> Self {
        Field::Array(Array::new(field))
    }

    /// Add a Map to a form
    ///
    /// # Example
    /// ```rust
    /// # use actix_form_data::{Form, Field};
    /// # fn main() {
    /// let form = Form::new()
    ///     .field(
    ///         "map-field",
    ///         Field::map()
    ///             .field("sub-field", Field::text())
    ///             .field("sub-field-two", Field::text())
    ///             .finalize()
    ///     );
    /// # }
    /// ```
    pub fn map() -> Map {
        Map::new()
    }

    fn valid_field(&self, name: VecDeque<&NamePart>) -> Option<FieldTerminator> {
        trace!("Checking {:?} and {:?}", self, name);
        match *self {
            Field::Array(ref arr) => arr.valid_field(name),
            Field::Map(ref map) => map.valid_field(name),
            Field::File(ref file_fn) => {
                if name.is_empty() {
                    Some(FieldTerminator::File(file_fn.clone()))
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
#[derive(Debug, Clone)]
pub struct Array {
    inner: Box<Field>,
}

impl Array {
    fn new(field: Field) -> Self {
        Array {
            inner: Box::new(field),
        }
    }

    fn valid_field(&self, mut name: VecDeque<&NamePart>) -> Option<FieldTerminator> {
        trace!("Checking {:?} and {:?}", self, name);
        match name.pop_front() {
            Some(name_part) => match name_part {
                NamePart::Array => self.inner.valid_field(name),
                _ => None,
            },
            None => None,
        }
    }
}

/// A definition of key-value pairs to be parsed from form data.
#[derive(Debug, Clone)]
pub struct Map {
    inner: Vec<(String, Field)>,
}

impl Map {
    fn new() -> Self {
        Map { inner: Vec::new() }
    }

    /// Add a `Field` to a map
    /// # Example
    /// ```rust
    /// # use actix_form_data::Field;
    /// #
    /// Field::map()
    ///     .field("sub-field", Field::text())
    ///     .field("sub-field-two", Field::text())
    ///     .finalize();
    /// ```
    pub fn field(mut self, key: &str, value: Field) -> Self {
        self.inner.push((key.to_owned(), value));

        self
    }

    /// Finalize the map into a `Field`, so it can be added to a Form
    /// ```rust
    /// # use actix_form_data::Field;
    /// #
    /// Field::map()
    ///     .field("sub-field", Field::text())
    ///     .field("sub-field-two", Field::text())
    ///     .finalize();
    /// ```
    pub fn finalize(self) -> Field {
        Field::Map(self)
    }

    fn valid_field(&self, mut name: VecDeque<&NamePart>) -> Option<FieldTerminator> {
        trace!("Checking {:?} and {:?}", self, name);
        match name.pop_front() {
            Some(name_part) => match name_part {
                NamePart::Map(part_name) => self
                    .inner
                    .iter()
                    .find(|&&(ref item, _)| *item == *part_name)
                    .and_then(|&(_, ref field)| field.valid_field(name)),
                _ => None,
            },
            None => None,
        }
    }
}

/// A structure that defines the fields expected in form data
///
/// # Example
/// ```rust
/// # use actix_form_data::{Form, Field};
/// let form = Form::new()
///     .field("field-name", Field::text())
///     .field("second-field", Field::int())
///     .field("third-field", Field::float())
///     .field("fifth-field", Field::file(|_, _, _| async move {
///         Ok(None) as Result<_, std::convert::Infallible>
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
#[derive(Clone)]
pub struct Form {
    pub(crate) max_fields: u32,
    pub(crate) max_field_size: usize,
    pub(crate) max_files: u32,
    pub(crate) max_file_size: usize,
    pub(crate) transform_error:
        Option<Arc<dyn Fn(crate::error::Error) -> actix_web::Error + Send + Sync>>,
    inner: Map,
}

impl Form {
    /// Create a new form
    ///
    /// If you wish to provide your own executor, use the `with_executor` method.
    ///
    /// Default values are as follows
    /// - max_fields: 100
    /// - max_field_size: 10_000 bytes
    /// - max_files: 20
    /// - max_files_size: 10_000_000 bytes
    pub fn new() -> Self {
        Form {
            max_fields: 100,
            max_field_size: 10_000,
            max_files: 20,
            max_file_size: 10_000_000,
            transform_error: None,
            inner: Map::new(),
        }
    }

    /// Add an optional error handler to transform errors produced by the middleware
    pub fn transform_error(
        mut self,
        f: impl Fn(crate::error::Error) -> actix_web::Error + Send + Sync + 'static,
    ) -> Self {
        self.transform_error = Some(Arc::new(f));

        self
    }

    /// Set the maximum number of fields allowed in the upload
    ///
    /// The upload will error if too many fields are provided.
    pub fn max_fields(mut self, max: u32) -> Self {
        self.max_fields = max;

        self
    }

    /// Set the maximum size of a field (in bytes)
    ///
    /// The upload will error if a provided field is too large.
    pub fn max_field_size(mut self, max: usize) -> Self {
        self.max_field_size = max;

        self
    }

    /// Set the maximum number of files allowed in the upload
    ///
    /// THe upload will error if too many files are provided.
    pub fn max_files(mut self, max: u32) -> Self {
        self.max_files = max;

        self
    }

    /// Set the maximum size for files (in bytes)
    ///
    /// The upload will error if a provided file is too large.
    pub fn max_file_size(mut self, max: usize) -> Self {
        self.max_file_size = max;

        self
    }

    pub fn field(mut self, name: &str, field: Field) -> Self {
        self.inner = self.inner.field(name, field);

        self
    }

    pub(crate) fn valid_field(&self, name: VecDeque<&NamePart>) -> Option<FieldTerminator> {
        self.inner.valid_field(name.clone())
    }
}

impl fmt::Debug for Form {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Form").field("inner", &self.inner).finish()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ContentDisposition {
    pub name: Option<String>,
    pub filename: Option<String>,
}

impl ContentDisposition {
    pub(crate) fn empty() -> Self {
        ContentDisposition {
            name: None,
            filename: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum NamePart {
    Map(String),
    Array,
}

impl NamePart {
    pub fn is_map(&self) -> bool {
        match *self {
            NamePart::Map(_) => true,
            _ => false,
        }
    }
}

#[derive(Clone)]
pub(crate) enum FieldTerminator {
    File(FileFn),
    Bytes,
    Int,
    Float,
    Text,
}

impl fmt::Debug for FieldTerminator {
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

pub(crate) type MultipartHash = (Vec<NamePart>, MultipartContent);
pub(crate) type MultipartForm = Vec<MultipartHash>;

#[derive(Debug)]
pub(crate) enum MultipartContent {
    File(FileMeta),
    Bytes(Bytes),
    Text(String),
    Int(i64),
    Float(f64),
}
