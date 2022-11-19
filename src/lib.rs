mod model;
mod node;
mod output;
mod relation;
mod serve;
mod watch;

use {
    crate::{node::Node, relation::Relation},
    clap::Parser,
    handlebars::RenderError,
    std::{ffi::OsString, io, net::SocketAddr, path::PathBuf},
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

pub async fn run<A, S>(args: A) -> Result<()>
where
    A: IntoIterator<Item = S>,
    S: Into<OsString> + Clone,
{
    let args = Args::parse_from(args);

    if let Err(err) = output::build(&args.model, &args.template, &args.output) {
        if !args.serve {
            return Err(err);
        }
        eprintln!("starting process error: {err:?}");
    }

    if !args.serve {
        return Ok(());
    }

    watch::watch(&args.model, &args.template, &args.output, {
        let model = args.model.clone();
        let template = args.template.clone();
        let output = args.output.clone();
        move || output::build(&model, &template, &output)
    });
    serve::serve(&args.output, &args.addr).await;
    Ok(())
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("node has unknown parent (node, parent): {list:?}")]
    NodeHasUnknownParent { list: Vec<(String, String)> },

    #[error("node has no definition: {0:?}")]
    NodeHasNoDefinition(Node),

    #[error("node relation has unknown parent (node, relation, parent): {list:?}")]
    NodeRelationHasUnknownParent { list: Vec<(String, String, String)> },

    #[error("relation has no definition: {0:?}")]
    RelationHasNoDefinition(Relation),

    #[error("render error: {source:?}")]
    RenderError {
        #[from]
        source: RenderError,
    },

    #[error("io error: {source:?}")]
    Io {
        #[from]
        source: io::Error,
    },

    #[error("yaml error: {source:?}")]
    Yaml {
        #[from]
        source: serde_yaml::Error,
    },
}

impl std::convert::From<Error> for RenderError {
    fn from(err: Error) -> Self {
        RenderError::from_error(&err.to_string(), err)
    }
}
