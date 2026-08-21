use std::env;
use std::ffi::OsStr;
use std::io::{self, Read, Write};
use std::path::Path;

use comrak;
use failure;
use html_escape;
use rst_parser;
use rst_renderer;

fn to_io(e: failure::Error) -> io::Error {
	io::Error::new(io::ErrorKind::Other, e.to_string())
}

fn md<W: Write>(buf: &str, out: &mut W) -> io::Result<()> {
	let s = comrak::markdown_to_html(buf, &comrak::ComrakOptions::default());
	out.write(s.as_bytes()).map(|_| ())
}

fn rst<W: Write>(buf: &str, out: &mut W) -> io::Result<()> {
	let d = rst_parser::parse(buf).map_err(to_io)?;
	rst_renderer::render_html(&d, out, false).map_err(to_io)
}

fn cat<W: Write>(buf: &str, out: &mut W) -> io::Result<()> {
	out.write(buf.as_bytes()).map(|_| ())
}

fn txt<W: Write>(buf: &str, out: &mut W) -> io::Result<()> {
	out.write(b"<pre>")?;
	html_escape::encode_safe_to_writer(buf, out)?;
	out.write(b"</pre>").map(|_| ())
}

fn main() -> io::Result<()> {
	let encoder = env::args().nth(1).as_ref()
		.map(Path::new).and_then(Path::extension).and_then(OsStr::to_str)
		.map(|s| match s.to_lowercase().as_str() {
			"md" | "mkd" | "markdown" | "mdown"  => md,
			"rst" => rst,
			"html" | "htm" => cat,
			_ => txt
		}).unwrap_or(txt);
	let mut buf = String::new();
	
	io::stdin().read_to_string(&mut buf)?;
	
	encoder(&buf.as_str(), &mut io::stdout())
}
