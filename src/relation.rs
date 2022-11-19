use {
    crate::{output::Merge, Error, Result},
    handlebars::Handlebars,
    serde::{Deserialize, Serialize},
    std::collections::HashSet,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    pub(crate) parent: Option<String>,
    pub(crate) tags: Option<HashSet<String>>,
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
        if self.tags.is_none() {
            self.tags = parent.tags.clone();
        } else if parent.tags.is_some() {
            self.tags = Some(self.tags.as_ref().unwrap() | parent.tags.as_ref().unwrap());
        }
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
    pub(crate) fn render_definition(&mut self, handlebars: &Handlebars) -> Result<()> {
        self.definition = Some(
            handlebars.render_template(
                self.definition
                    .as_ref()
                    .ok_or_else(|| Error::RelationHasNoDefinition(self.clone()))?,
                self,
            )?,
        );
        Ok(())
    }
}
