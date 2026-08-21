use std::env;
use std::ffi::OsStr;
use std::io::{self, Read, Write};
use std::path::Path;

use comrak::{markdown_to_html, ComrakOptions};
use html_escape;
use rst_parser;
use rst_renderer;

fn md(buf: &str, out: &mut impl Write) -> io::Result<()> {
	out.write_all(markdown_to_html(buf, &ComrakOptions::default()).as_bytes())
}

fn rst(buf: &str, out: &mut impl Write) -> io::Result<()> {
	rst_parser::parse(buf)
		.and_then(|d| rst_renderer::render_html(&d, out, false))
		.map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
}

fn cat(buf: &str, out: &mut impl Write) -> io::Result<()> {
	out.write_all(buf.as_bytes())
}

fn txt(buf: &str, out: &mut impl Write) -> io::Result<()> {
	out.write_all(b"<pre>")?;
	html_escape::encode_safe_to_writer(buf, out)?;
	out.write_all(b"</pre>")
}

fn main() -> io::Result<()> {
	let mut buf = String::new();
	io::stdin().read_to_string(&mut buf)?;

	(env::args().nth(1).as_ref()
		.map(Path::new)
		.and_then(Path::extension)
		.and_then(OsStr::to_str)
		.map(|s| match s.to_lowercase().as_str() {
			"md" | "mkd" | "markdown" | "mdown" => md,
			"rst" => rst,
			"html" | "htm" => cat,
			_ => txt
		})
		.unwrap_or(txt)
	)(&buf.as_str(), &mut io::stdout())
}
