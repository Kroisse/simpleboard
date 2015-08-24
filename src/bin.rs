extern crate handlebars_iron;
extern crate iron;
extern crate logger;
extern crate mount;
extern crate plugin;
extern crate postgres;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate rustc_serialize;
extern crate staticfile;

mod middleware;
mod model;

use std::collections::BTreeMap;
use std::iter::FromIterator;
use std::path::Path;
use std::sync::Arc;

use handlebars_iron::{HandlebarsEngine, Template};
use handlebars_iron::Watchable;
use iron::{Chain, Iron, IronResult, Plugin, Request, Response, Set};
use iron::status;
use logger::Logger;
use mount::Mount;
use r2d2_postgres::PostgresConnectionManager;
use rustc_serialize::json::{Json, ToJson};
use staticfile::Static;

use middleware::db;
use model::Post;

type Conn = db::Connection<PostgresConnectionManager>;

fn list_posts(req: &mut Request) -> IronResult<Response> {
    let conn = req.get_ref::<Conn>().unwrap();
    let stmt = conn.prepare("SELECT title, body FROM posts").unwrap();
    let mut posts = vec![];
    for row in stmt.query(&[]).unwrap() {
        posts.push(Post {
            title: row.get("title"),
            body: row.get("body"),
        });
    }
    let data: BTreeMap<_, _> = FromIterator::from_iter(vec![
        ("posts".to_owned(), Json::Array(posts.into_iter().map(|e| e.to_json()).collect())),
    ]);
    Ok(Response::with(Template::new("list_posts", data)).set(status::Ok))
}

fn main() {
    let mut chain = Chain::new({
        let mut mount = Mount::new();
        mount.mount("/static", Static::new(Path::new("./static")));
        mount.mount("/", list_posts);
        mount
    });

    let manager = PostgresConnectionManager::new("postgresql://gracie@localhost/simpleboard",
                                                 postgres::SslMode::None).unwrap();
    chain.link_before(db::ConnectionPool::new(manager).unwrap());
    let template_engine_ref = Arc::new(HandlebarsEngine::new("./templates", ".hbs"));
    template_engine_ref.watch();
    chain.link_after(template_engine_ref);
    chain.link(Logger::new(None));

    let server = Iron::new(chain);
    server.http("0.0.0.0:8404").unwrap();
}
