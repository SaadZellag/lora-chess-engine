use actix_web::{web, HttpResponse};

pub async fn index() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../../static/index.html"))
}

pub async fn static_file(path: web::Path<String>) -> HttpResponse {
    let path = path.into_inner();
    
    match path.as_str() {
        "app.js" => HttpResponse::Ok()
            .content_type("application/javascript")
            .body(include_str!("../../static/app.js")),
        "style.css" => HttpResponse::Ok()
            .content_type("text/css")
            .body(include_str!("../../static/style.css")),
        _ => HttpResponse::NotFound().finish(),
    }
}
