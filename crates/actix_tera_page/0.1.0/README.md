# actix_tera_page

This crate provides a middleware for `actix_web` that reduces the boilerplate needed to
create SSR websites with `Tera`. It matches GET request paths to templates and renders them
using a shared "base context". An example use case would be populating a website navbar
with user information or login/signup buttons, depending on if there is a user logged in or not.