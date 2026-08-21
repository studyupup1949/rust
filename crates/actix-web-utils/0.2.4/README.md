# Actix-web-utils

by franklin blanco

Adds some useful structs, enums, macros, etc... to your typical web app, only compatible with actix-web backends for the moment.

## Why use this crate?

Honestly, I find myself repeating the same structs, behaviours and logic in my backend applications for the web apps I develop.

This crate just makes it all uniform and readable.

## How?

Mainly by standarizing everything I can, repeating logic & code everywhere possible to maintain a uniform & straightforward development.

Most web app backends have a Controller (Where routes are defined), Service (Where bsuiness logic lives (can also be inside the controller)), Repository/dao (Database access logic & methods), Clients/Communicators (Api consumption). Let's start with the controller.

### Controller

You usually have a defined route with a desired return type and an error type. 


```rust
use actix_web::{http::header::ContentType, HttpResponse};

async fn index() -> HttpResponse {
    HttpResponse::Ok()
        .content_type(ContentType::plaintext())
        // .insert_header(("X-Hdr", "sample")) <-- This line is irrelevant for this example
        .body("data")
}
```
*Taken Straight from actix.rs*

The way actix sells this seems a bit dry to me. How about this:

```rust
use actix_web::{http::header::ContentType, HttpResponse};
use actix_web_utils::extensions::{TypedHttpResponse};

async fn index() -> TypedHttpResponse<String> {
    TypedHttpResponse::return_standard_response(200, "data".to_string())
}
```

Much better for readability, and this wraps everything you give it inside a Json response, as is usually the standard. You can see the status code outright, and the response has to be a String if successful (this is important for later). 

Sure, you loose a lot of the modularity that actix web gives you. But that's why it's not a replacement for HttpResponse, but another type. You can always go back and use HttpResponse, my library isn't made to replace it, but to reduce the repetition it brings.

#### Intended Limitations: 
- Can only use things that implement serde::Serialize because it attempts to wrap everything inside a json. This is planned.
- Modularity is lost, no response headers, most of the features of HttpResponse get lost. This is also planned.
- No custom errors. using this means you will be using the Error types & MessageResource defined in this package. Sorry about that, coding a modular solution for everybody would be hard. Feel free to contribute, really. I'll reply asap. In case you don't want to contribute also feel free to fork.

### Service 
This is where your business logic lives. This is really the most variable of all, as everyone has their own way of doing things. But I have seen many cases of code where it's just this:

```rust
fn service_layer_function() -> HttpResponse {
	// ... Some Business logic
	let value_returned_from_match = match function_that_returns_a_result() {
		Ok(value) => value,
		Err(error) => return 
		HttpResponse::BadRequest().json(web::Json(error.to_string()))
	};
	// ... More Business logic
}
```

When you could be doing this: 

```rust
fn service_layer_function() -> TypedHttpResponse<T> { //T can be whatever you want
	// ... Some Business logic
	let value_returned_from_match = unwrap_or_return_handled_error!(
		function_that_returns_a_result()
	);
	// ... More Business logic
}
```

Much better and more readable for me.

### Repository (Database Access Object)
**UNFINISHED!**

#### MessageResource

A key-message error type to return back a neat, well formatted error to a frontend. The key is optional, as I won't force you to use internationalization.

```json
{
	"key": "ERROR_KEY",
	"message": "This is the part where you put the error message."
}
```

This is supposed to be sent back to the client, so the client can display the error to the user, and/or get the translation (with the key). It is optional.

#### TypedHttpResponse

A wrapper for HttpResponse. Contains a type specification inside so that you can visualize and expect a certain type.

Said type can only be a Serializable (Json).

It can also return a MessageResource or nothing at all.