use diesel::{BoolExpressionMethods, Expression, ExpressionMethods};

/// Generates a filter for Diesel that looks for a record in the table by either id or username.
///
/// # Example
/// ```rs
/// let filter = get_identifier_filter(users::id, users::username, "3"); // SELECT * FROM users WHERE id=3 or username='3'
/// let filter = get_identifier_filter(users::id, users::username, "smokingplaya"); // SELECT * FROM users WHERE id=-1 or username='smokingplaya'
/// ```
pub fn get_identifier_filter<T, U>(
  id_column: T,
  username_column: U,
  identifier: String
) -> diesel::dsl::Or<diesel::dsl::Eq<T, i32>, diesel::dsl::Eq<U, String>>
where
  T: Expression<SqlType = diesel::sql_types::Integer> + Copy,
  U: Expression<SqlType = diesel::sql_types::Text> + Copy,
{
  // короче в чём прикол
  // если у вас в бд будет запись с id -1, то оно вернёт эту запись при попытке найти запись по юзернейму
  // но если у вас в бд запись с id -1 то вы долбаеб скорее всего
  // так что not mine problem
  let id_value = identifier.parse::<i32>().ok().unwrap_or(-1);
  id_column.eq(id_value).or(username_column.eq(identifier))
}
