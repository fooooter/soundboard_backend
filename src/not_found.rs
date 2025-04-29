use drain_macros::*;

#[drain_endpoint("not_found")]
pub fn not_found() {
    let content: Vec<u8> = Vec::from(r#"
    <!DOCTYPE html>
    <html lang="en">
        <head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>404</title>
        </head>
        <body>
            <h2>404 Not Found</h2>
        </body>
    </html>"#
    );

    set_header!("Content-Type", "text/html; charset=utf-8");

    Some(content)
}