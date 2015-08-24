use std::iter::FromIterator;

use rustc_serialize::json::{Json, ToJson};

pub struct Post {
    pub title: String,
    pub body: String
}

impl ToJson for Post {
    fn to_json(&self) -> Json {
        Json::Object(FromIterator::from_iter(vec![
            ("title".to_owned(), self.title.to_json()),
            ("body".to_owned(), self.body.to_json()),
        ]))
    }
}
