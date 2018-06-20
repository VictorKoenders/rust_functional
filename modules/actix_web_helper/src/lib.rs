extern crate actix_web;

pub fn get_property(req: &actix_web::HttpRequest, field: &str) -> String {
    req.match_info().get(field).unwrap_or("").to_string()
}

pub fn format1(format: &str, arg0: impl std::fmt::Display) -> String {
    format.replace("{}", &format!("{}", arg0))
}
