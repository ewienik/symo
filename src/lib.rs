mod model;
mod node;
mod output;
mod relation;
mod serve;
mod watch;

pub use crate::{model::Model, node::Node, relation::Relation};

use std::{net::SocketAddr, path::Path};

pub fn run_one_time(model: &Path, template: &Path, output: &Path) -> Result<()> {
    output::build(model, template, output)
}

pub async fn run_serve(
    model: &Path,
    template: &Path,
    output: &Path,
    addr: &SocketAddr,
) -> Result<()> {
    watch::watch(model, template, output, {
        let model = model.to_path_buf();
        let template = template.to_path_buf();
        let output = output.to_path_buf();
        move || output::build(&model, &template, &output)
    });
    serve::serve(output, addr).await
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
        source: handlebars::RenderError,
    },

    #[error("io error: {source:?}")]
    Io {
        #[from]
        source: std::io::Error,
    },

    #[error("yaml error: {source:?}")]
    Yaml {
        #[from]
        source: serde_yaml::Error,
    },

    #[error("hyper error: {source:?}")]
    Hyper {
        #[from]
        source: hyper::Error,
    },
}

impl std::convert::From<Error> for handlebars::RenderError {
    fn from(err: Error) -> Self {
        handlebars::RenderError::from_error(&err.to_string(), err)
    }
}
