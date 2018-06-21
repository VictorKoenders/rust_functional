extern crate actix_web;
extern crate serde;
extern crate serde_json;

pub fn get_property(req: &actix_web::HttpRequest, field: &str) -> String {
    req.match_info().get(field).unwrap_or("").to_string()
}

pub fn jsonify(obj: &impl serde::Serialize) -> String {
    serde_json::to_string(obj).unwrap()
}
