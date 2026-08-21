use crate::domain::users::user_email::UserEmail;
use crate::domain::users::user_name::UserName;
use crate::domain::users::user_password::UserPassword;

pub struct NewUser {
    pub username: UserName,
    pub email: UserEmail,
    pub password: UserPassword,
}
