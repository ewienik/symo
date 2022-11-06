use {
    crate::{node::Node, relationship::Relationship, Merge},
    serde::{Deserialize, Serialize},
    std::{
        collections::{BTreeMap, HashSet},
        ffi::OsString,
        fs::File,
        path::Path,
    },
    walkdir::WalkDir,
};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Model {
    #[serde(default)]
    pub(crate) relationships: BTreeMap<String, Relationship>,
    #[serde(default)]
    pub(crate) nodes: BTreeMap<String, Node>,
    #[serde(default)]
    pub(crate) diagrams: BTreeMap<String, String>,
}

fn merge<T: Merge>(map: &mut BTreeMap<String, T>) {
    let mut done: HashSet<_> = map
        .iter()
        .filter(|(_, value)| value.parent().is_none())
        .map(|(name, _)| name.clone())
        .collect();
    while done.len() < map.len() {
        let outstanding: BTreeMap<_, _> = map
            .iter()
            .filter(|(_, child)| {
                if let Some(parent) = child.parent() {
                    done.contains(&parent)
                } else {
                    false
                }
            })
            .map(|(name, child)| (name.clone(), (*child).clone()))
            .collect();
        if outstanding.len() == 0 {
            map.iter()
                .filter(|(_, node)| node.parent().is_some())
                .for_each(|(name, node)| {
                    println!(
                        "Node {} has unknown parent {}",
                        name,
                        node.parent().unwrap()
                    )
                });
            panic!("There are unknown parents in nodes!");
        }
        outstanding.into_iter().for_each(|(name, mut child)| {
            let parent = child.parent().unwrap();
            child.merge(map.get(&parent).unwrap());
            map.insert(name.clone(), child);
            done.insert(name);
        });
    }
}

impl Model {
    pub(crate) fn new(path: &Path) -> Self {
        let mut model: Self = WalkDir::new(path)
            .into_iter()
            .filter_map(|item| item.ok())
            .filter(|item| item.file_type().is_file())
            .filter(|item| item.path().extension().unwrap_or(&OsString::new()) == "yaml")
            .map(|item| item.into_path())
            .fold(
                Self {
                    relationships: BTreeMap::new(),
                    nodes: BTreeMap::new(),
                    diagrams: BTreeMap::new(),
                },
                |mut acc, item| {
                    let mut model: Self =
                        serde_yaml::from_reader(File::open(item).unwrap()).unwrap();
                    acc.relationships.append(&mut model.relationships);
                    acc.nodes.append(&mut model.nodes);
                    acc.diagrams.append(&mut model.diagrams);
                    acc
                },
            );
        merge(&mut model.relationships);
        merge(&mut model.nodes);
        model.nodes = model
            .nodes
            .clone()
            .into_iter()
            .map(|(id, mut node)| {
                node.id = Some(id.clone());
                node.merge_relationships(&id, &model);
                (id, node)
            })
            .collect();
        model.nodes = model
            .nodes
            .clone()
            .into_iter()
            .map(|(id, mut node)| {
                node.merge_relationships_with_parent(&id, &model);
                (id, node)
            })
            .collect();
        model
    }

    pub(crate) fn diagram_definitions(&self, diagram: &String) -> String {
        let relationships: HashSet<_> = self
            .diagrams
            .get(diagram)
            .unwrap()
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|id| self.nodes.contains_key(&*id))
            .collect();
        let mut sorted: Vec<_> = relationships.iter().collect();
        sorted.sort();
        sorted
            .into_iter()
            .map(|id| {
                let node = self.nodes.get(&*id).unwrap();
                let mut definitions: Vec<_> = node
                    .relationships
                    .iter()
                    .flat_map(|map| map.iter())
                    .filter(|(_, relationship)| {
                        let right = relationship.right.as_ref().unwrap();
                        relationships.contains(right) && self.nodes.contains_key(right)
                    })
                    .filter_map(|(_, relationship)| relationship.definition.as_ref())
                    .collect();
                definitions.sort();
                definitions
                    .iter()
                    .fold(node.definition.as_ref().unwrap().clone(), |acc, line| {
                        format!("{}\n{}", acc, line)
                    })
            })
            .fold(String::new(), |acc, line| format!("{}\n{}", acc, line))
    }
}
