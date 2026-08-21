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
use actix_web::web::BytesMut;
use std::{collections::HashMap, path::Path, rc::Rc};
use streem::IntoStreamer;
use tokio::task::JoinSet;
use tracing::Instrument;

fn consolidate<T>(mf: MultipartForm<T>) -> Value<T> {
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
        .try_fold(vec![], |mut v, part| {
            if v.is_empty() && !part.is_map() {
                return Err(Error::ContentDisposition);
            }

            v.push(part);
            Ok(v)
        })
}

fn parse_content_disposition(field: &actix_multipart::Field) -> ContentDisposition {
    let content_disposition = field.content_disposition();

    ContentDisposition {
        name: content_disposition.get_name().map(|v| v.to_string()),
        filename: content_disposition.get_filename().map(|v| v.to_string()),
    }
}

async fn handle_file_upload<T, E>(
    field: actix_multipart::Field,
    filename: Option<String>,
    form: &Form<T, E>,
    file_fn: &FileFn<T, E>,
) -> Result<Result<MultipartContent<T>, E>, Error>
where
    T: 'static,
    E: 'static,
{
    let filename = filename.ok_or(Error::Filename)?;
    let path: &Path = filename.as_ref();

    let filename = path.file_name().and_then(|filename| filename.to_str());

    let filename = filename.ok_or(Error::Filename)?.to_owned();

    let content_type = field.content_type().cloned();

    let max_file_size = form.max_file_size;

    let field_stream = streem::try_from_fn(move |yielder| async move {
        let mut file_size = 0;

        let mut stream = field.into_streamer();

        while let Some(bytes) = stream.try_next().await? {
            tracing::trace!("Bytes from field");

            file_size += bytes.len();

            if file_size > max_file_size {
                drop(bytes);

                while stream.try_next().await?.is_some() {
                    tracing::trace!("Dropping oversized bytes");
                }

                return Err(Error::FileSize);
            }

            yielder.yield_ok(bytes).await;
        }

        tracing::debug!("Finished consuming field");

        Ok(())
    });

    let result = file_fn(
        filename.clone(),
        content_type.clone(),
        Box::pin(field_stream),
    )
    .await;

    match result {
        Ok(result) => Ok(Ok(MultipartContent::File(FileMeta {
            filename,
            content_type,
            result,
        }))),
        Err(e) => Ok(Err(e)),
    }
}

async fn handle_form_data<'a, T, E>(
    field: actix_multipart::Field,
    term: FieldTerminator<'a, T, E>,
    form: &Form<T, E>,
) -> Result<MultipartContent<T>, Error>
where
    T: 'static,
    E: 'static,
{
    tracing::trace!("In handle_form_data, term: {:?}", term);
    let mut buf = Vec::new();

    let mut stream = field.into_streamer();

    while let Some(bytes) = stream.try_next().await? {
        tracing::trace!("bytes from field");

        if buf.len() + bytes.len() > form.max_field_size {
            drop(buf);

            while stream.try_next().await?.is_some() {
                tracing::trace!("Dropping oversized bytes");
            }

            return Err(Error::FieldSize);
        }

        buf.push(bytes);
    }

    let bytes = match buf.len() {
        0 => return Err(Error::FieldSize),
        1 => buf.pop().expect("contains an element"),
        _ => {
            let total_length = buf.iter().map(|b| b.len()).sum();

            let mut bytes = BytesMut::with_capacity(total_length);

            for b in buf {
                bytes.extend(b);
            }

            bytes.freeze()
        }
    };

    tracing::debug!("Finished consuming field");

    if let FieldTerminator::Bytes = term {
        return Ok(MultipartContent::Bytes(bytes));
    }

    let s = std::str::from_utf8(&bytes).map_err(Error::ParseField)?;

    match term {
        FieldTerminator::Bytes | FieldTerminator::File(_) => Err(Error::FieldType),
        FieldTerminator::Text => Ok(MultipartContent::Text(String::from(s))),
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

async fn handle_stream_field<T, E>(
    field: actix_multipart::Field,
    form: Rc<Form<T, E>>,
) -> Result<Result<MultipartHash<T>, E>, Error>
where
    T: 'static,
    E: 'static,
{
    let content_disposition = parse_content_disposition(&field);

    let name = content_disposition.name.ok_or(Error::Field)?;
    let name = parse_multipart_name(name)?;

    let term = form
        .valid_field(name.iter().collect())
        .ok_or(Error::FieldType)?;

    let content = match term {
        FieldTerminator::File(file_fn) => {
            match handle_file_upload(field, content_disposition.filename, &form, file_fn).await? {
                Ok(content) => content,
                Err(e) => return Ok(Err(e)),
            }
        }
        term => handle_form_data(field, term, &form).await?,
    };

    Ok(Ok((name, content)))
}

/// Handle multipart streams from Actix Web
#[tracing::instrument(level = "TRACE", skip_all)]
pub async fn handle_multipart<T, E>(
    m: actix_multipart::Multipart,
    form: Rc<Form<T, E>>,
) -> Result<Result<Value<T>, E>, Error>
where
    T: 'static,
    E: 'static,
{
    let mut multipart_form = Vec::new();
    let mut file_count: u32 = 0;
    let mut field_count: u32 = 0;

    let mut set = JoinSet::new();

    let mut m = m.into_streamer();

    let mut error: Option<Error> = None;
    let mut provided_error: Option<E> = None;
    let mut is_closed = false;
    let mut stream_error = false;

    'outer: loop {
        tracing::trace!("multipart loop");

        if error.is_some() || provided_error.is_some() {
            set.abort_all();

            if !stream_error {
                while let Some(res) = m.next().await {
                    tracing::trace!("draining multipart field");

                    if let Ok(field) = res {
                        let mut stream = field.into_streamer();
                        while stream.next().await.is_some() {
                            tracing::trace!("Throwing away uploaded bytes, we have an error");
                        }
                    } else {
                        break;
                    }
                }
            }

            while set.join_next().await.is_some() {
                tracing::trace!("Throwing away joined result");
            }

            break 'outer;
        }

        tokio::select! {
            opt = m.next(), if !is_closed => {
                tracing::trace!("Selected stream");
                is_closed = opt.is_none();

                if let Some(res) = opt {
                    match res {
                        Ok(field) => {
                            set.spawn_local(handle_stream_field(field, Rc::clone(&form)).instrument(tracing::trace_span!("multipart-field")));
                        },
                        Err(e) => {
                            is_closed = true;
                            stream_error = true;
                            error = Some(e.into());
                            continue 'outer;
                        }
                    }
                }
            }
            opt = set.join_next(), if !set.is_empty() => {
                tracing::trace!("Selected set");
                if let Some(res) = opt {
                    let (name_parts, content) = match res {
                        Ok(Ok(Ok(tup))) => tup,
                        Ok(Ok(Err(e))) => {
                            provided_error = Some(e);
                            continue 'outer;
                        }
                        Ok(Err(e)) => {
                            error = Some(e);
                            continue 'outer;
                        },
                        Err(e) => {
                            error = Some(e.into());
                            continue 'outer;
                        },
                    };

                    let (l, r) = match count(&content, file_count, field_count, &form) {
                        Ok(tup) => tup,
                        Err(e) => {
                            error = Some(e);
                            continue 'outer;
                        }
                    };

                    file_count = l;
                    field_count = r;

                    multipart_form.push((name_parts, content));
                }
            }
            else => {
                break 'outer;
            }
        }
    }

    tracing::debug!("Finished consuming multipart");

    if let Some(e) = provided_error {
        return Ok(Err(e));
    }

    if let Some(e) = error {
        return Err(e);
    }

    Ok(Ok(consolidate(multipart_form)))
}

fn count<T, E>(
    content: &MultipartContent<T>,
    mut file_count: u32,
    mut field_count: u32,
    form: &Form<T, E>,
) -> Result<(u32, u32), Error> {
    match content {
        MultipartContent::File(_) => {
            file_count += 1;
            if file_count > form.max_files {
                return Err(Error::FileCount);
            }
        }
        _ => {
            field_count += 1;
            if field_count > form.max_fields {
                return Err(Error::FieldCount);
            }
        }
    }

    Ok((file_count, field_count))
}
