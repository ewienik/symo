use {
    crate::{model::Model, Result},
    handlebars::{Context, Handlebars, Helper, HelperResult, Output, RenderContext},
    serde::Serialize,
    std::{
        collections::{BTreeMap, HashSet},
        ffi::OsString,
        fs::{self, File},
        io::Write,
        iter,
        path::Path,
        sync::Arc,
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

fn render_nodes_definitions(handlebars: &Handlebars, model: &mut Model) -> Result<()> {
    model
        .nodes
        .iter_mut()
        .map(|(_, node)| node)
        .try_for_each(|node| node.render_definition(handlebars))
}

fn render_nodes_relations_definitions(handlebars: &Handlebars, model: &mut Model) -> Result<()> {
    model
        .nodes
        .iter_mut()
        .filter_map(|(_, node)| node.relations.as_mut())
        .flat_map(|relations| relations.iter_mut())
        .flat_map(|(_, relations)| relations.iter_mut())
        .try_for_each(|relation| relation.render_definition(handlebars))
}

fn render_diagrams(
    mut handlebars: Handlebars,
    mut model: Model,
) -> Result<BTreeMap<String, String>> {
    render_nodes_definitions(&handlebars, &mut model)?;
    render_nodes_relations_definitions(&handlebars, &mut model)?;

    let model = Arc::new(model);
    handlebars.register_helper(
        "definitions",
        Box::new({
            let model = Arc::clone(&model);
            move |h: &Helper,
                  _r: &Handlebars,
                  ctx: &Context,
                  _rc: &mut RenderContext,
                  out: &mut dyn Output|
                  -> HelperResult {
                let tags: HashSet<_> = h
                    .params()
                    .iter()
                    .filter_map(|v| v.relative_path())
                    .map(|v| v.to_string())
                    .collect();
                let name = ctx
                    .data()
                    .as_object()
                    .unwrap()
                    .get("diagram-name")
                    .unwrap()
                    .as_str()
                    .unwrap();

                out.write(&model.diagram_definitions(name, tags)?)?;
                Ok(())
            }
        }),
    );

    Ok(model
        .diagrams
        .iter()
        .map(|(name, definition)| {
            (
                name.clone(),
                handlebars
                    .render_template(
                        definition,
                        &iter::once(("diagram-name".to_string(), name.clone()))
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
        .collect())
}

fn process<T>(
    handlebars: &Handlebars,
    data: &T,
    output: &Path,
    template: &Path,
    src: &Path,
) -> Result<()>
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
    handlebars.render_template_to_write(
        std::str::from_utf8(&fs::read(src).unwrap()).unwrap(),
        data,
        dst,
    )?;
    Ok(())
}

pub(crate) fn build(model: &Path, template: &Path, output: &Path) -> Result<()> {
    let data = render_diagrams(new_handlebars(), Model::new(model)?)?;
    let handlebars = new_handlebars();
    WalkDir::new(template)
        .into_iter()
        .filter_map(|item| item.ok())
        .filter(|item| item.file_type().is_file())
        .filter(|item| item.path().extension().unwrap_or(&OsString::new()) == "md")
        .filter(|item| !item.path().ancestors().any(|path| path == output))
        .try_for_each(|item| process(&handlebars, &data, output, template, item.path()))?;
    Ok(())
}
