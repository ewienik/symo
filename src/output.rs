use {
    crate::model::Model,
    handlebars::Handlebars,
    serde::Serialize,
    std::{
        collections::BTreeMap,
        ffi::OsString,
        fs::{self, File},
        io::Write,
        iter,
        path::Path,
    },
    walkdir::WalkDir,
};

pub(crate) trait Merge: Clone {
    fn parent(&self) -> Option<String>;
    fn merge(&mut self, parent: &Self);
}

fn new_handlebars<'a>() -> Handlebars<'a> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(true);
    handlebars
}

fn new_data(handlebars: &Handlebars, mut model: Model) -> BTreeMap<String, String> {
    model
        .nodes
        .iter_mut()
        .filter_map(|(_, node)| {
            node.render_definition(handlebars);
            node.relationships.as_mut()
        })
        .flat_map(|relationships| relationships.iter_mut())
        .for_each(|(_, relationship)| relationship.render_definition(handlebars));
    model
        .diagrams
        .iter()
        .map(|(name, definition)| {
            (
                name.clone(),
                handlebars
                    .render_template(
                        definition,
                        &iter::once(("definitions".to_string(), model.diagram_definitions(name)))
                            .chain(model.nodes.iter().filter_map(|(id, node)| {
                                node.name
                                    .as_ref()
                                    .map(|name| (format!("{}-name", id), format!("\"{}\"", name)))
                            }))
                            .collect::<BTreeMap<_, _>>(),
                    )
                    .unwrap(),
            )
        })
        .collect()
}

fn process<T>(handlebars: &Handlebars, data: &T, output: &Path, template: &Path, src: &Path)
where
    T: Serialize,
{
    let dst = output.join(src.strip_prefix(template).unwrap());
    println!(
        "rendering {}...",
        dst.strip_prefix(output).unwrap().to_str().unwrap()
    );
    let mut dst = File::create(dst).unwrap();
    dst.write_all(b"<!-- DO NOT EDIT; Autogenerated -->\n\n")
        .unwrap();
    handlebars
        .render_template_to_write(
            std::str::from_utf8(&fs::read(src).unwrap()).unwrap(),
            data,
            dst,
        )
        .unwrap();
}

pub fn build(model: &Path, template: &Path, output: &Path) {
    let handlebars = new_handlebars();
    let data = new_data(&handlebars, Model::new(model));
    WalkDir::new(template)
        .into_iter()
        .filter_map(|item| item.ok())
        .filter(|item| item.file_type().is_file())
        .filter(|item| item.path().extension().unwrap_or(&OsString::new()) == "md")
        .filter(|item| !item.path().ancestors().any(|path| path == output))
        .for_each(|item| process(&handlebars, &data, output, template, item.path()));
}
