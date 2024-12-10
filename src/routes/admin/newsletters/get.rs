use actix_web::http::header::ContentType;
use actix_web::HttpResponse;
use actix_web_flash_messages::IncomingFlashMessages;
use std::fmt::Write;

pub async fn get_newletters_form(
    flash_messages: IncomingFlashMessages,
) -> Result<HttpResponse, actix_web::Error> {
    let mut msg_html = String::new();
    for m in flash_messages.iter() {
        writeln!(msg_html, "<p><i>{}</i></p>", m.content()).unwrap();
    }

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            r#"<!DOCTYPE html>
        <html lang="en">
        <head>
            <meta http-equiv="content-type" content="text/html; charset=utf-8">
            <title>Send Newsletter</title>
        </head>
        <body>
            {msg_html}  
            <form action="/admin/newsletters" method="post">
                <label>Title
                    <input
                    type="text"
                    placeholder="Enter title"
                    name="title"
                    >
                </label>
                <br>
                <label>Plain text content:<br>
                    <textarea
                        placeholder="Enter the content in plain text"
                        name="content_text"
                        rows="20"
                        cols="50"
                    ></textarea>
                </label>
                <br>
                <label>HTML content:<br>
                    <textarea
                        placeholder="Enter the content in HTML format"
                        name="content_html"
                        rows="20"
                        cols="50"
                    ></textarea>
                </label>
                <br>
                <button type="submit">Publish</button>
            </form>
            <p><a href="/admin/dashboard">&lt;- Back</a></p>
        </body>
        </html>"#,
        )))
}
