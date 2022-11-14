use {
    crate::{model::Model, output::Merge, relation::Relation},
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
    pub(crate) relations: Option<BTreeMap<String, Relation>>,
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
    pub(crate) fn merge_relations(&mut self, id: &str, model: &Model) {
        if let Some(relations) = &mut self.relations {
            relations
                .iter_mut()
                .filter(|(_, child)| child.parent.is_some())
                .filter_map(|(id_relation, child)| {
                    if let Some(parent) = model.relations.get(&child.parent().unwrap()) {
                        Some((id_relation, child, parent))
                    } else {
                        println!("Unknown parent for node {} relation {}", id, id_relation);
                        None
                    }
                })
                .for_each(|(id_relation, child, parent)| {
                    child.left = Some(id.to_string());
                    child.right = Some(id_relation.clone());
                    child.merge(parent);
                });
        };
    }

    pub(crate) fn merge_relations_with_parent(&mut self, id: &str, model: &Model) {
        let parent_relations = self
            .parent()
            .into_iter()
            .filter_map(|parent| model.nodes.get(&parent))
            .filter_map(|parent| parent.relations.as_ref())
            .flatten();
        if let Some(relations) = &mut self.relations {
            parent_relations.for_each(|(id_relation, parent)| {
                if let Some(child) = relations.get_mut(id_relation) {
                    child.left = Some(id.to_string());
                    child.merge(parent);
                } else {
                    relations.insert(id_relation.clone(), parent.clone());
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
                .context(format!("failed render definition for {:?} relation", self))
                .unwrap(),
        );
    }
}
