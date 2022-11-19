use {
    crate::{model::Model, output::Merge, relation::Relation, Error, Result},
    handlebars::Handlebars,
    serde::{Deserialize, Serialize},
    std::collections::BTreeMap,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: Option<String>,
    pub parent: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub technology: Option<String>,
    pub relations: Option<BTreeMap<String, Vec<Relation>>>,
    pub definition: Option<String>,
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
    pub(crate) fn merge_relations(&mut self, id: &str, model: &Model) -> Result<()> {
        let mut errors = vec![];
        if let Some(relations) = &mut self.relations {
            relations
                .iter_mut()
                .flat_map(|(id_relation, child)| {
                    child
                        .iter_mut()
                        .map(|child| (id_relation.to_owned(), child))
                })
                .filter(|(_, child)| child.parent.is_some())
                .filter_map(|(id_relation, child)| {
                    let id_parent = child
                        .parent()
                        .unwrap_or_else(|| "(not defined)".to_string());
                    if let Some(parent) = model.relations.get(&id_parent) {
                        Some((id_relation, child, parent))
                    } else {
                        errors.push((id.to_owned(), id_relation, id_parent));
                        None
                    }
                })
                .for_each(|(id_relation, child, parent)| {
                    child.left = Some(id.to_string());
                    child.right = Some(id_relation);
                    child.merge(parent);
                });
        };
        if errors.is_empty() {
            Ok(())
        } else {
            Err(Error::NodeRelationHasUnknownParent { list: errors })
        }
    }

    pub(crate) fn merge_relations_with_parent(&mut self, id: &str, model: &Model) {
        let parent_relations = self
            .parent()
            .into_iter()
            .filter_map(|parent| model.nodes.get(&parent))
            .filter_map(|parent| parent.relations.as_ref())
            .flatten();
        if let Some(relations) = &mut self.relations {
            parent_relations
                .flat_map(|(id_relation, parent)| {
                    parent.iter().map(|parent| (id_relation.to_owned(), parent))
                })
                .for_each(|(id_relation, parent)| {
                    if let Some(children) = relations.get_mut(&id_relation) {
                        if let Some(child) = children.iter_mut().find(|child| {
                            child
                                .parent
                                .as_ref()
                                .map(|relation_parent| relation_parent == &id_relation)
                                .unwrap_or(false)
                        }) {
                            child.left = Some(id.to_string());
                            child.merge(parent);
                        } else {
                            children.push(parent.clone());
                        }
                    } else {
                        relations.insert(id_relation.clone(), vec![parent.clone()]);
                    }
                });
        };
    }

    pub(crate) fn render_definition(&mut self, handlebars: &Handlebars) -> Result<()> {
        self.definition = Some(
            handlebars.render_template(
                &self
                    .definition
                    .as_ref()
                    .ok_or_else(|| Error::NodeHasNoDefinition(self.clone()))?
                    .clone(),
                self,
            )?,
        );
        Ok(())
    }
}
