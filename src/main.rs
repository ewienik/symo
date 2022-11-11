use std::env;

#[tokio::main]
async fn main() {
    symo::run(env::args_os()).await;
}
