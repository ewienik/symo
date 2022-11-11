use {
    notify::RecursiveMode,
    notify_debouncer_mini,
    std::{fs, path::Path, sync::mpsc, time::Duration},
};

pub(crate) fn watch(model: &Path, template: &Path, output: &Path, job: impl Fn() + Send + 'static) {
    let model = fs::canonicalize(model).unwrap();
    let template = fs::canonicalize(template).unwrap();
    let output = fs::canonicalize(output).unwrap();
    std::thread::spawn(move || {
        let (tx, rx) = mpsc::channel();

        let mut debouncer =
            notify_debouncer_mini::new_debouncer(Duration::from_secs(1), None, tx).unwrap();

        debouncer
            .watcher()
            .watch(&model, RecursiveMode::Recursive)
            .unwrap();
        debouncer
            .watcher()
            .watch(&template, RecursiveMode::Recursive)
            .unwrap();

        rx.iter().for_each(|e| {
            if e.unwrap_or(Vec::new())
                .iter()
                .map(|e| &e.path)
                .find(|path| {
                    path.ancestors()
                        .take_while(|path| *path != model && *path != template)
                        .find(|path| *dbg!(path) == output)
                        .is_none()
                })
                .is_some()
            {
                job();
            }
        });
    });
}
