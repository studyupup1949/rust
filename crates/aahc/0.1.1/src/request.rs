mod body;
mod headers;

pub use body::Send as SendBody;
pub use headers::send as send_headers;

/// Metadata about a request.
///
/// Certain information about a request is needed in order to properly process the corresponding
/// response. In order to decouple request and response processing, that information is packed into
/// a `Metadata` structure and returned to the application. The application must then pass the
/// `Metadata` structure back in when preparing to receive the response.
///
/// This type does not have any public API. The application cannot examine or modify its contents.
/// It is intended to be stashed somewhere when the application finishes sending the request and
/// then passed, unmodified, when the application begins receiving the response.
///
/// In the simple case of straight-line code with no pipelining, the object can simply be
/// immediately passed into [`receive_headers`](super::receive_headers). More complex situations
/// may call for more complex handling; for example, if pipelining is in use, the application
/// likely needs to maintain a FIFO queue of `Metadata` structures, each being pushed to the queue
/// when the request finishes and popped from the queue when ready to read the corresponding
/// response. Or, if it is expected that the server may take some time to start generating a
/// response, the application may wish to free most of the resources associated with the request
/// and “park” the socket until the response begins arriving; in such a situation, the `Metadata`
/// structure must be stored along with the socket and is very lightweight.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Metadata {
	/// Whether the method was `HEAD`.
	pub(crate) head: bool,

	/// Whether there was a `Connection` header with the value `close`.
	pub(crate) connection_close: bool,
}
