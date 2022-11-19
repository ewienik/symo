use {
    crate::Result,
    axum::{
        http::{Request, Response, StatusCode},
        response::IntoResponse,
        routing, Router,
    },
    std::{
        fs,
        future::{self, Ready},
        iter,
        net::SocketAddr,
        path::{Path, PathBuf},
        str::FromStr,
        task::{Context, Poll},
    },
    tower::Service,
    tower_http::services::ServeDir,
};

async fn handle_error<E>(_: E) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}

#[derive(Clone)]
struct DirListingService {
    base: PathBuf,
}

impl DirListingService {
    fn new(base: &Path) -> Self {
        Self {
            base: base.canonicalize().unwrap(),
        }
    }
}

impl<ReqBody> Service<Request<ReqBody>> for DirListingService
where
    ReqBody: Send + 'static,
{
    type Response = Response<String>;
    type Error = std::io::Error;
    type Future = Ready<std::result::Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<std::result::Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: Request<ReqBody>) -> Self::Future {
        future::ready(
            if let Some(path) = &self
                .base
                .join(PathBuf::from_str(&request.uri().path()[1..]).unwrap())
                .canonicalize()
                .ok()
                .iter()
                .filter(|path| (path).is_dir())
                .find(|path| path.ancestors().any(|path| (*path) == self.base))
            {
                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .body(
                        iter::once(format!(
                            "<html>\n<meta>\n<title>Directory {}</title>\n</meta>\n<body>\n<ul>",
                            request.uri().path()
                        ))
                        .chain(
                            iter::once("<li><a href=\"..\">..</a></li>\n".to_string())
                                .filter(|_| request.uri().path() != "/"),
                        )
                        .chain(
                            fs::read_dir(path)
                                .unwrap()
                                .filter_map(|path| path.ok())
                                .filter_map(|path| {
                                    path.file_name().to_str().map(|name| name.to_string())
                                })
                                .map(|name| format!("<li><a href=\"{name}\">{name}</a></li>\n")),
                        )
                        .chain(iter::once("</ul>\n</body>\n</html>\n".to_string()))
                        .collect(),
                    )
                    .unwrap())
            } else {
                Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(String::new())
                    .unwrap())
            },
        )
    }
}

pub(crate) async fn serve(path: &Path, addr: &SocketAddr) -> Result<()> {
    let server = axum::Server::bind(addr).serve(
        Router::new()
            .fallback(
                routing::get_service(
                    ServeDir::new(path)
                        .append_index_html_on_directories(false)
                        .fallback(DirListingService::new(path)),
                )
                .handle_error(handle_error),
            )
            .into_make_service(),
    );
    println!("listening on http://{}", server.local_addr());

    server.await?;
    Ok(())
}
