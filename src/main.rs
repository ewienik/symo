use {
    clap::Parser,
    std::{env, net::SocketAddr, path::PathBuf},
};

#[derive(Parser)]
#[clap(about, version)]
struct Args {
    model: PathBuf,
    template: PathBuf,
    output: PathBuf,

    #[clap(short, long)]
    serve: bool,

    #[clap(short, long, default_value = "127.0.0.1:0")]
    addr: SocketAddr,
}

#[tokio::main]
async fn main() {
    let args = Args::parse_from(env::args_os());

    if args.serve {
        symo::run_serve(&args.model, &args.template, &args.output, &args.addr).await
    } else {
        symo::run_one_time(&args.model, &args.template, &args.output)
    }
    .unwrap()
}
