use {
    crate::{model::Model, output::Merge, relationship::Relationship},
    anyhow::Context,
    handlebars::Handlebars,
    serde::{Deserialize, Serialize},
    std::collections::BTreeMap,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Node {
    pub(crate) id: Option<String>,
    pub(crate) parent: Option<String>,
    pub(crate) name: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) technology: Option<String>,
    pub(crate) relationships: Option<BTreeMap<String, Relationship>>,
    pub(crate) definition: Option<String>,
}

impl Merge for Node {
    fn parent(&self) -> Option<String> {
        self.parent.clone()
    }

    fn merge(&mut self, parent: &Self) {
        if self.name.is_none() {
            self.name = parent.name.clone();
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

impl Node {
    pub(crate) fn merge_relationships(&mut self, id: &str, model: &Model) {
        if let Some(relationships) = &mut self.relationships {
            relationships
                .iter_mut()
                .filter(|(_, child)| child.parent.is_some())
                .filter_map(|(id_relationship, child)| {
                    if let Some(parent) = model.relationships.get(&child.parent().unwrap()) {
                        Some((id_relationship, child, parent))
                    } else {
                        println!(
                            "Unknown parent for node {} relationship {}",
                            id, id_relationship
                        );
                        None
                    }
                })
                .for_each(|(id_relationship, child, parent)| {
                    child.left = Some(id.to_string());
                    child.right = Some(id_relationship.clone());
                    child.merge(parent);
                });
        };
    }

    pub(crate) fn merge_relationships_with_parent(&mut self, id: &str, model: &Model) {
        let parent_relationships = self
            .parent()
            .into_iter()
            .filter_map(|parent| model.nodes.get(&parent))
            .filter_map(|parent| parent.relationships.as_ref())
            .flatten();
        if let Some(relationships) = &mut self.relationships {
            parent_relationships.for_each(|(id_relationship, parent)| {
                if let Some(child) = relationships.get_mut(id_relationship) {
                    child.left = Some(id.to_string());
                    child.merge(parent);
                } else {
                    relationships.insert(id_relationship.clone(), parent.clone());
                }
            });
        };
    }

    pub(crate) fn render_definition(&mut self, handlebars: &Handlebars) {
        self.definition = Some(
            handlebars
                .render_template(
                    &self
                        .definition
                        .as_ref()
                        .context(format!("no definition for {:?}", self))
                        .unwrap()
                        .clone(),
                    self,
                )
                .context(format!(
                    "failed render definition for {:?} relationship",
                    self
                ))
                .unwrap(),
        );
    }
}
