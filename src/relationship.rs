use {
    crate::Merge,
    anyhow::Context,
    handlebars::Handlebars,
    serde::{Deserialize, Serialize},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Relationship {
    pub(crate) parent: Option<String>,
    pub(crate) left: Option<String>,
    pub(crate) right: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) technology: Option<String>,
    pub(crate) definition: Option<String>,
}

impl Merge for Relationship {
    fn parent(&self) -> Option<String> {
        self.parent.clone()
    }

    fn merge(&mut self, parent: &Self) {
        if let None = self.right {
            self.right = parent.right.clone();
        }
        if let None = self.description {
            self.description = parent.description.clone();
        }
        if let None = self.technology {
            self.technology = parent.technology.clone();
        }
        if let None = self.definition {
            self.definition = parent.definition.clone();
        }
    }
}

impl Relationship {
    pub(crate) fn render_definition(&mut self, handlebars: &Handlebars) {
        self.definition = Some(
            handlebars
                .render_template(
                    &self
                        .definition
                        .as_ref()
                        .context(format!("no definition for {:?}", self))
                        .unwrap(),
                    self,
                )
                .context(format!("failed render definition for {:?}", self))
                .unwrap(),
        )
    }
}
