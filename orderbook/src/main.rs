use actix_web::{web, App, HttpServer, Responder, HttpResponse};

async fn hello() -> impl Responder {
    "Hello, world!"
}

async fn print_content(body: String) -> impl Responder {
    println!("Received content: {}", body);
    HttpResponse::Ok().body("Content printed to console")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    


    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(hello))
            .route("/print", web::post().to(print_content))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

