use {
    crate::output::Merge,
    anyhow::Context,
    handlebars::Handlebars,
    serde::{Deserialize, Serialize},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Relation {
    pub(crate) parent: Option<String>,
    pub(crate) left: Option<String>,
    pub(crate) right: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) technology: Option<String>,
    pub(crate) definition: Option<String>,
}

impl Merge for Relation {
    fn parent(&self) -> Option<String> {
        self.parent.clone()
    }

    fn merge(&mut self, parent: &Self) {
        if self.right.is_none() {
            self.right = parent.right.clone();
        }
        if self.description.is_none() {
            self.description = parent.description.clone();
        }
        if self.technology.is_none() {
            self.technology = parent.technology.clone();
        }
        if self.definition.is_none() {
            self.definition = parent.definition.clone();
        }
    }
}

impl Relation {
    pub(crate) fn render_definition(&mut self, handlebars: &Handlebars) {
        self.definition = Some(
            handlebars
                .render_template(
                    self.definition
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
