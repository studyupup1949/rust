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

use crate::{
    error::Error,
    types::{
        ContentDisposition, FieldTerminator, FileFn, FileMeta, Form, MultipartContent,
        MultipartForm, MultipartHash, NamePart, Value,
    },
};
use bytes::BytesMut;
use futures::{
    select,
    stream::{FuturesUnordered, StreamExt},
};
use std::{
    collections::HashMap,
    path::Path,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use tracing::trace;

fn consolidate(mf: MultipartForm) -> Value {
    mf.into_iter().fold(
        Value::Map(HashMap::new()),
        |mut acc, (mut nameparts, content)| {
            let start_value = Value::from(content);

            nameparts.reverse();
            let value = nameparts
                .into_iter()
                .fold(start_value, |acc, namepart| match namepart {
                    NamePart::Map(name) => {
                        let mut hm = HashMap::new();

                        hm.insert(name, acc);

                        Value::Map(hm)
                    }
                    NamePart::Array => Value::Array(vec![acc]),
                });

            acc.merge(value);
            acc
        },
    )
}

fn parse_multipart_name(name: String) -> Result<Vec<NamePart>, Error> {
    name.split('[')
        .map(|part| {
            if part.len() == 1 && part.ends_with(']') {
                NamePart::Array
            } else if part.ends_with(']') {
                NamePart::Map(part.trim_end_matches(']').to_owned())
            } else {
                NamePart::Map(part.to_owned())
            }
        })
        .fold(Ok(vec![]), |acc, part| match acc {
            Ok(mut v) => {
                if v.len() == 0 && !part.is_map() {
                    return Err(Error::ContentDisposition);
                }

                v.push(part);
                Ok(v)
            }
            Err(e) => Err(e),
        })
}

fn parse_content_disposition(field: &actix_multipart::Field) -> ContentDisposition {
    match field.content_disposition() {
        Some(x) => ContentDisposition {
            name: x.get_name().map(|v| v.to_string()),
            filename: x.get_filename().map(|v| v.to_string()),
        },
        None => ContentDisposition::empty(),
    }
}

async fn handle_file_upload(
    field: actix_multipart::Field,
    filename: Option<String>,
    form: Form,
    file_fn: FileFn,
) -> Result<MultipartContent, Error> {
    let filename = filename.ok_or(Error::Filename)?;
    let path: &Path = filename.as_ref();

    let filename = path.file_name().and_then(|filename| filename.to_str());

    let filename = filename.ok_or(Error::Filename)?.to_owned();

    let file_size = Arc::new(AtomicUsize::new(0));

    let content_type = field.content_type().clone();

    let saved_as = file_fn(
        filename.clone(),
        content_type.clone(),
        Box::pin(field.then(move |res| {
            let form = form.clone();
            let file_size = file_size.clone();
            async move {
                match res {
                    Ok(bytes) => {
                        let size = file_size.fetch_add(bytes.len(), Ordering::Relaxed);

                        if size + bytes.len() > form.max_file_size {
                            return Err(Error::FileSize);
                        }

                        Ok(bytes)
                    }
                    Err(e) => Err(Error::from(e)),
                }
            }
        })),
    )
    .await?;

    Ok(MultipartContent::File(FileMeta {
        filename,
        content_type,
        saved_as,
    }))
}

async fn handle_form_data(
    mut field: actix_multipart::Field,
    term: FieldTerminator,
    form: Form,
) -> Result<MultipartContent, Error> {
    trace!("In handle_form_data, term: {:?}", term);
    let mut bytes = BytesMut::new();

    while let Some(res) = field.next().await {
        let b = res?;
        if bytes.len() + b.len() > form.max_field_size {
            return Err(Error::FieldSize);
        }

        bytes.extend(b);
    }

    if let FieldTerminator::Bytes = term {
        return Ok(MultipartContent::Bytes(bytes.freeze()));
    }

    let s = String::from_utf8(bytes.to_vec()).map_err(Error::ParseField)?;

    match term {
        FieldTerminator::Bytes | FieldTerminator::File(_) => {
            return Err(Error::FieldType);
        }
        FieldTerminator::Text => Ok(MultipartContent::Text(s)),
        FieldTerminator::Float => s
            .parse()
            .map_err(Error::ParseFloat)
            .map(MultipartContent::Float),
        FieldTerminator::Int => s
            .parse()
            .map_err(Error::ParseInt)
            .map(MultipartContent::Int),
    }
}

async fn handle_stream_field(
    field: actix_multipart::Field,
    form: Form,
) -> Result<MultipartHash, Error> {
    let content_disposition = parse_content_disposition(&field);

    let name = content_disposition.name.ok_or(Error::Field)?;
    let name = parse_multipart_name(name)?;

    let term = form
        .valid_field(name.iter().collect())
        .ok_or(Error::FieldType)?;

    let content = match term {
        FieldTerminator::File(file_fn) => {
            handle_file_upload(field, content_disposition.filename, form, file_fn).await?
        }
        term => handle_form_data(field, term, form.clone()).await?,
    };

    Ok((name, content))
}

/// Handle multipart streams from Actix Web
pub async fn handle_multipart(m: actix_multipart::Multipart, form: Form) -> Result<Value, Error> {
    let mut multipart_form = Vec::new();
    let mut file_count: u32 = 0;
    let mut field_count: u32 = 0;

    let mut unordered = FuturesUnordered::new();

    let mut m = m.fuse();

    loop {
        select! {
            opt = m.next() => {
                if let Some(res) = opt {
                    unordered.push(handle_stream_field(res?, form.clone()));
                }
            }
            opt = unordered.next() => {
                if let Some(res) = opt {
                    let (name_parts, content) = res?;

                    let (l, r) = count(&content, file_count, field_count, &form)?;
                    file_count = l;
                    field_count = r;

                    multipart_form.push((name_parts, content));
                }
            }
            complete => break,
        }
    }

    Ok(consolidate(multipart_form))
}

fn count(
    content: &MultipartContent,
    mut file_count: u32,
    mut field_count: u32,
    form: &Form,
) -> Result<(u32, u32), Error> {
    match content {
        MultipartContent::File(_) => {
            file_count += 1;
            if file_count >= form.max_files {
                return Err(Error::FileCount);
            }
        }
        _ => {
            field_count += 1;
            if field_count >= form.max_fields {
                return Err(Error::FieldCount);
            }
        }
    }

    Ok((file_count, field_count))
}
