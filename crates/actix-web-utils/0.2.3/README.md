## Actix-web-utils
by franklin blanco

Adds some useful structs to your typical web app, only compatible with actix-web.

##### MessageResource
A key-message error type to return back a neat, well formatted error to a frontend. The key is optional, as I won't force you to use internationalization.

#### TypedHttpResponse
A wrapper for HttpResponse. Contains a type specification inside so that you can visualize and expect a certain type. 
Said type can only be a Serializable (Json).
It can also return a MessageResource or nothing at all.