#![allow(unused)]
//! demo from https://github.com/joelparkerhenderson/demo-rust-axum

use std::collections::HashMap;
use std::net::SocketAddr;

use axum::{Router, routing::get};

use http::request;
use serde_json::{Value, json};

/// axum handler for "GET /" which returns a string and causes axum to
/// immediately respond with status code `200 OK` and with the string.
pub async fn hello() -> String {
    "Hello, World!".into()
}

/// axum handler for any request that fails to match the router routes.
/// This implementation returns HTTP status code Not Found (404).
pub async fn fallback(uri: axum::http::Uri) -> impl axum::response::IntoResponse {
    (
        axum::http::StatusCode::NOT_FOUND,
        format!("No route {}", uri),
    )
}

/// Tokio signal handler that will wait for a user to press CTRL+C.
/// We use this in our hyper `Server` method `with_graceful_shutdown`.
async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("expect tokio signal ctrl-c");
    println!("signal shutdown");
}

/// axum handler for "GET /demo.html" which responds with HTML text.
/// The `Html` type sets an HTTP header content-type of `text/html`.
pub async fn get_demo_html() -> axum::response::Html<&'static str> {
    "<h1>Hello</h1>".into()
}

/// axum handler that responds with typical HTML coming from a file.
/// This uses the Rust macro `std::include_str` to include a UTF-8 file
/// path, relative to `main.rs`, as a `&'static str` at compile time.
async fn hello_html() -> axum::response::Html<&'static str> {
    include_str!("hello.html").into()
}

/// axum handler for "GET /demo-status" which returns a HTTP status
/// code, such as OK (200), and a custom user-visible string message.
pub async fn demo_status() -> (axum::http::StatusCode, String) {
    (axum::http::StatusCode::OK, "Everything is OK".to_string())
}

/// axum handler for "GET /demo-uri" which shows the request's own URI.
/// This shows how to write a handler that receives the URI.
pub async fn demo_uri(uri: axum::http::Uri) -> String {
    format!("The URI is: {:?}", uri)
}

/// axum handler for "GET /demo.png" which responds with an image PNG.
/// This sets a header "image/png" then sends the decoded image data.
async fn get_demo_png() -> impl axum::response::IntoResponse {
    let png = concat!(
        "iVBORw0KGgoAAAANSUhEUgAAAAEAAAAB",
        "CAYAAAAfFcSJAAAADUlEQVR42mPk+89Q",
        "DwADvgGOSHzRgAAAAABJRU5ErkJggg=="
    );
    (
        axum::response::AppendHeaders([(axum::http::header::CONTENT_TYPE, "image/png")]),
        base64::decode(png).unwrap(),
    )
}

/// axum handler for "GET /foo" which returns a string message.
/// This shows our naming convention for HTTP GET handlers.
pub async fn get_foo() -> String {
    "GET foo".to_string()
}

/// axum handler for "PUT /foo" which returns a string message.
/// This shows our naming convention for HTTP PUT handlers.
pub async fn put_foo() -> String {
    "PUT foo".to_string()
}

/// axum handler for "PATCH /foo" which returns a string message.
/// This shows our naming convention for HTTP PATCH handlers.
pub async fn patch_foo() -> String {
    "PATCH foo".to_string()
}

/// axum handler for "POST /foo" which returns a string message.
/// This shows our naming convention for HTTP POST handlers.
pub async fn post_foo() -> String {
    "POST foo".to_string()
}

/// axum handler for "DELETE /foo" which returns a string message.
/// This shows our naming convention for HTTP DELETE handlers.
pub async fn delete_foo() -> String {
    "DELETE foo".to_string()
}

/// axum handler for "GET /items/:id" which uses `axum::extract::Path`.
/// This extracts a path parameter then deserializes it as needed.
pub async fn get_items_id(axum::extract::Path(id): axum::extract::Path<u32>) -> String {
    format!("Get items with id: {:?}", id)
}

/// axum handler for "GET /items" which uses `axum::extract::Query`.
/// This extracts query parameters and creates a key-value pair map.
pub async fn get_items(
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> String {
    format!("Get items with query params: {:?}", params)
}

/// axum handler for "PUT /demo.json" which uses `axum::extract::Json`.
/// This buffers the request body then deserializes it bu using serde.
/// The `Json` type supports types that implement `serde::Deserialize`.
pub async fn get_demo_json() -> axum::extract::Json<Value> {
    json!({"a":"b"}).into()
}

/// axum handler for "PUT /demo.json" which uses `axum::extract::Json`.
/// This buffers the request body then deserializes it using serde.
/// The `Json` type supports types that implement `serde::Deserialize`.
pub async fn put_demo_json(
    axum::extract::Json(data): axum::extract::Json<serde_json::Value>,
) -> String {
    format!("Put demo JSON data: {:?}", data)
}

use data::DATA;

/// Use Thread for spawning a thread e.g. to acquire our DATA mutex lock.
use std::thread;

/// To access data, create a thread, spawn it, then get the lock.
/// When you're done, then join the thread with its parent thread.
async fn print_data() {
    thread::spawn(move || {
        let data = DATA.lock().unwrap();
        println!("data: {:?}", data);
    })
    .join()
    .unwrap()
}

use book::Book;

/// axum handler for "GET /books" which responds with a resource page.
/// This demo uses our DATA; a production app could use a database.
/// This demo must clone the DATA in order to sort items by title.
pub async fn get_books() -> axum::response::Html<String> {
    thread::spawn(move || {
        let data = DATA.lock().unwrap();
        let mut books = data.values().collect::<Vec<_>>().clone();
        books.sort_by(|a, b| a.title.cmp(&b.title));
        books
            .iter()
            .map(|&book| format!("<p>{}</p>\n", &book))
            .collect::<String>()
    })
    .join()
    .unwrap()
    .into()
}

/// axum handler for "GET /books/:id" which responds with one resource HTML page.
/// This demo app uses our DATA variable, and iterates on it to find the id.
pub async fn get_books_id(
    axum::extract::Path(id): axum::extract::Path<u32>,
) -> axum::response::Html<String> {
    thread::spawn(move || {
        let data = DATA.lock().unwrap();
        match data.get(&id) {
            Some(book) => format!("<p>{}</p>\n", &book),
            None => format!("<p>Book id {} not found</p>", id),
        }
    })
    .join()
    .unwrap()
    .into()
}

/// axum handler for "PUT /books" which creates a new book resource.
/// This demo shows how axum can extract JSON data into a Book struct.
pub async fn put_books(
    axum::extract::Json(book): axum::extract::Json<Book>,
) -> axum::response::Html<String> {
    thread::spawn(move || {
        let mut data = DATA.lock().unwrap();
        data.insert(book.id, book.clone());
        format!("Put book: {}", &book)
    })
    .join()
    .unwrap()
    .into()
}

/// axum handler for "GET /books/:id/form" which responds with a form.
/// This demo shows how to write a typical HTML form with input fields.
pub async fn get_books_id_form(
    axum::extract::Path(id): axum::extract::Path<u32>,
) -> axum::response::Html<String> {
    thread::spawn(move || {
        let data = DATA.lock().unwrap();
        match data.get(&id) {
            Some(book) => format!(
                concat!(
                    "<form method=\"post\" action=\"/books/{}/form\">\n",
                    "<input type=\"hidden\" name=\"id\" value=\"{}\">\n",
                    "<p><input name=\"title\" value=\"{}\"></p>\n",
                    "<p><input name=\"author\" value=\"{}\"></p>\n",
                    "<input type=\"submit\" value=\"Save\">\n",
                    "</form>\n"
                ),
                &book.id, &book.id, &book.title, &book.author
            ),
            None => format!("<p>Book id {} not found</p>", id),
        }
    })
    .join()
    .unwrap()
    .into()
}

/// axum handler for "POST /books/:id/form" which submits an HTML form.
/// This demo shows how to do a form submission then update a resource.
pub async fn post_books_id_form(form: axum::extract::Form<Book>) -> axum::response::Html<String> {
    let new_book: Book = form.0;
    thread::spawn(move || {
        let mut data = DATA.lock().unwrap();
        if data.contains_key(&new_book.id) {
            data.insert(new_book.id, new_book.clone());
            format!("<p>{}</p>\n", &new_book)
        } else {
            format!("Book id not found: {}", &new_book.id)
        }
    })
    .join()
    .unwrap()
    .into()
}

/// axum handler for "DELETE /books/:id" which destroys a resource.
/// This demo extracts an id, then mutates the book in the DATA store.
pub async fn delete_books_id(
    axum::extract::Path(id): axum::extract::Path<u32>,
) -> axum::response::Html<String> {
    thread::spawn(move || {
        let mut data = DATA.lock().unwrap();
        if data.contains_key(&id) {
            data.remove(&id);
            format!("Delete book id: {}", &id)
        } else {
            format!("Book id not found: {}", &id)
        }
    })
    .join()
    .unwrap()
    .into()
}
/// Use tracing crates for application-level tracing output.
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use axum::{async_trait, extract::FromRequestParts};
use axum_extra::handler::HandlerCallWithExtractors;

// handlers for varying levels of access
async fn admin(admin: AdminPermissions) {
    // request came from an admin
    dbg!(admin);
}

async fn user(user: User) {
    // we have a `User`
    dbg!(user);
}

async fn guest() {
    // `AdminPermissions` and `User` failed, so we're just a guest
    dbg!("guest");
}

// extractors for checking permissions
#[derive(Debug)]
struct AdminPermissions {}

#[async_trait]
impl<S> FromRequestParts<S> for AdminPermissions
where
    S: Send + Sync,
{
    type Rejection = String;

    // check for admin permissions...
    async fn from_request_parts(
        parts: &mut request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        Err("admin auth not implemented".into())
    }
}

#[derive(Debug)]
struct User {}

#[async_trait]
impl<S> FromRequestParts<S> for User
where
    S: Send + Sync,
{
    type Rejection = String;

    // check for a logged in user...
    async fn from_request_parts(
        parts: &mut request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        Err("user auth not implemented".into())
    }
}

#[tokio::main]
async fn main() {
    let host = [127, 0, 0, 1];
    let port = 8080;
    let addr = SocketAddr::from((host, port));
    // Start tracing.
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();
    // Build our application with a single route.
    let app = Router::new()
        .fallback(fallback)
        .route("/", get(hello))
        .route("/demo.html", get(get_demo_html))
        .route("/hello.html", get(hello_html))
        .route("/demo-status", get(demo_status))
        .route("/demo-uri", get(demo_uri))
        .route("/demo.png", get(get_demo_png))
        .route(
            "/foo",
            get(get_foo)
                .put(put_foo)
                .patch(patch_foo)
                .post(post_foo)
                .delete(delete_foo),
        )
        .route("/items/:id", get(get_items_id))
        .route("/items", get(get_items))
        .route("/demo.json", get(get_demo_json).put(put_demo_json))
        .route("/books", get(get_books).put(put_books))
        .route("/books/:id", get(get_books_id).delete(delete_books_id))
        .route(
            "/books/:id/form",
            get(get_books_id_form).post(post_books_id_form),
        )
        .route(
            "/users/:id",
            get(
                // first try `admin`, if that rejects run `user`, finally falling back
                // to `guest`
                admin.or(user).or(guest),
            ),
        );

    // Run our application as a hyper server on http://localhost:8080.
    // axum::Server::bind(&"127.0.0.1:8080".parse().unwrap())
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

mod book {
    /// Use Deserialize to convert e.g. from request JSON into Book struct.
    use serde::Deserialize;

    /// Demo book structure with some example fields for id, title, author.
    #[derive(Debug, Deserialize, Clone, Eq, Hash, PartialEq)]
    pub struct Book {
        pub id: u32,
        pub title: String,
        pub author: String,
    }
    /// Display the book using the format "{title} by {author}".
    /// This is a typical Rust trait and is not axum-specific.
    impl std::fmt::Display for Book {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "{} by {}", self.title, self.author)
        }
    }
}

mod data {

    /// Bring Book struct into scope
    use super::book::Book;
    /// Use once_cell for creating a global variable e.g. our DATA data.
    use once_cell::sync::Lazy;
    use std::collections::HashMap;

    /// Use Mutex for thread-safe access to a variable e.g. our DATA data.
    use std::sync::Mutex;

    /// Create a data store as a global variable with `Lazy` and `Mutex`.
    /// This demo implementation uses a `HashMap` for ease and speed.
    /// The map key is a primary key for lookup; the map value is a Book.
    pub static DATA: Lazy<Mutex<HashMap<u32, Book>>> = Lazy::new(|| {
        Mutex::new(HashMap::from([
            (
                1,
                Book {
                    id: 1,
                    title: "Antigone".into(),
                    author: "Sophocles".into(),
                },
            ),
            (
                2,
                Book {
                    id: 2,
                    title: "Beloved".into(),
                    author: "Toni Morrison".into(),
                },
            ),
            (
                3,
                Book {
                    id: 3,
                    title: "Candide".into(),
                    author: "Voltaire".into(),
                },
            ),
        ]))
    });
}
