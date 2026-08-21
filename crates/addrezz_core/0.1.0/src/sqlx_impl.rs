//! `sqlx` support for [`Addr`].
//!
//! `Addr` is stored as `TEXT`/`VARCHAR` in the database. On decode the
//! string is run back through [`Addr::parse`]. Works across any sqlx
//! backend that has a `String` type impl (Postgres, MySQL, SQLite).

use sqlx::{
    Database, Decode, Encode, Type,
    encode::IsNull,
    error::BoxDynError,
};

use crate::Addr;

impl<DB: Database> Type<DB> for Addr
where
    String: Type<DB>,
{
    fn type_info() -> DB::TypeInfo {
        <String as Type<DB>>::type_info()
    }

    fn compatible(ty: &DB::TypeInfo) -> bool {
        <String as Type<DB>>::compatible(ty)
    }
}

impl<'q, DB: Database> Encode<'q, DB> for Addr
where
    String: Encode<'q, DB>,
{
    fn encode_by_ref(
        &self,
        buf: &mut <DB as Database>::ArgumentBuffer,
    ) -> Result<IsNull, BoxDynError> {
        <String as Encode<'q, DB>>::encode_by_ref(&self.to_string(), buf)
    }
}

impl<'r, DB: Database> Decode<'r, DB> for Addr
where
    String: Decode<'r, DB>,
{
    fn decode(value: <DB as Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = <String as Decode<'r, DB>>::decode(value)?;
        Addr::parse(&s).map_err(Into::into)
    }
}
