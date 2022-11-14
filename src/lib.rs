mod model;
mod node;
mod output;
mod relation;
mod serve;
mod watch;

use {
    clap::Parser,
    std::{ffi::OsString, net::SocketAddr, path::PathBuf},
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

pub async fn run<A, S>(args: A)
where
    A: IntoIterator<Item = S>,
    S: Into<OsString> + Clone,
{
    let args = Args::parse_from(args);

    output::build(&args.model, &args.template, &args.output);

    if !args.serve {
        return;
    }

    watch::watch(&args.model, &args.template, &args.output, {
        let model = args.model.clone();
        let template = args.template.clone();
        let output = args.output.clone();
        move || output::build(&model, &template, &output)
    });
    serve::serve(&args.output, &args.addr).await;
}
