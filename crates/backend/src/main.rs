use std::path::Path;

use actix_cors::Cors;
use actix_files as fs;
use actix_web::{
    middleware::{self, DefaultHeaders},
    web,
    web::post,
    App, HttpResponse, HttpServer, Result,
};
use clap::Parser;

use backend::{route_compile, Opts};

pub struct FrontendState {
    pub frontend_folder: String,
}

pub fn route_frontend(at: &str, dir: &str) -> actix_files::Files {
    fs::Files::new(at, dir).index_file("index.html")
}

pub async fn route_frontend_version(data: web::Data<FrontendState>) -> Result<actix_files::NamedFile> {
    Ok(fs::NamedFile::open(
        Path::new(&data.frontend_folder).join("index.html"),
    )?)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let opts: Opts = Opts::parse();

    let port = opts.port;
    let host = opts.host.clone();

    if let Some(path) = &opts.frontend_folder {
        if !Path::new(path).is_dir() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Frontend folder not found: {}", path),
            ));
        }
    }

    async fn health() -> HttpResponse {
        HttpResponse::Ok().finish()
    }

    HttpServer::new(move || {
        let opts: Opts = opts.clone();
        let frontend_folder = opts.frontend_folder.clone();

        let mut app = App::new()
            .service(web::resource("/health").to(health))
            // Enable GZIP compression
            .wrap(middleware::Compress::default())
            .wrap(
                DefaultHeaders::new()
                    .add(("Cross-Origin-Opener-Policy", "same-origin"))
                    .add(("Cross-Origin-Embedder-Policy", "require-corp")),
            )
            .wrap(
                Cors::default()
                        .allow_any_origin()
                        // .allowed_methods(vec!["GET", "POST", "OPTIONS"])
                        .allow_any_method()
                        // .allowed_headers(vec!["Content-Type", "Authorization"])
                        .allow_any_header()
                        .max_age(3600),
                )
            .route("/compile", post().to(|body| route_compile(body)));

        // Serve frontend files if configured via CLI
        match frontend_folder {
            Some(path) => {
                app = app
                    .app_data(web::Data::new(FrontendState {
                        frontend_folder: path.clone(),
                    }))
                    .route("/v{tail:.*}", web::get().to(route_frontend_version))
                    .service(route_frontend("/", path.as_ref()));
            },
            None => {
                println!(
                    "Warning: Starting backend without serving static frontend files due to missing configuration."
                )
            },
        }

        app
    })
    .bind(format!("{}:{}", &host, &port))?
    .run()
    .await?;

    Ok(())
}
