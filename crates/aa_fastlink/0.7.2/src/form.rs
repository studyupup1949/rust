use warp::reply::{Reply, html};

pub fn render_form() -> impl Reply {
    html(
        r#"
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>Fast Download</title>
                <style>
                    body { font-family: sans-serif; max-width: 600px; margin: 2rem auto; padding: 0 1rem; }
                    form { display: flex; gap: 0.5rem; }
                    input[type="text"] { flex: 1; padding: 0.5rem; font-size: 1rem; }
                    input[type="submit"] { padding: 0.5rem 1rem; font-size: 1rem; }
                </style>
            </head>
            <body>
                <h1>Fast Download</h1>
                <form action="/dl" method="post">
                    <label for="link">Enter eBook URL or MD5:</label>
                    <input
                        type="text"
                        id="link"
                        name="link"
                        placeholder="Paste URL or 32-character hash here"
                        autocomplete="off"
                        required
                    >
                    <input type="submit" value="Download">
                </form>
            </body>
        </html>
        "#,
    )
}
