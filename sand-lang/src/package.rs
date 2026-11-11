//! Loading a package: the root file and every `mod` it declares.
//!
//! Module files follow Rust's layout. A root file or a `mod.sand` file owns
//! its directory, so `mod combat;` there loads `combat.sand` or
//! `combat/mod.sand` next to it. Any other file `combat.sand` owns the
//! directory `combat/`, so `mod hits;` inside it loads `combat/hits.sand`.

use crate::ast::{Item, ItemKind, SourceFile, Span};
use crate::diagnostic::Diagnostic;
use crate::parser;
use crate::source::{FileId, FileProvider, LineIndex};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The conventional root file name a package directory is discovered by.
pub const ROOT_FILE: &str = "pack.sand";

/// One parsed source file.
#[derive(Debug, Clone)]
pub struct SourceData {
    /// Path the file was loaded from.
    pub path: PathBuf,
    /// Source text.
    pub text: String,
    /// Position index for the text.
    pub lines: LineIndex,
    /// Parsed syntax tree.
    pub ast: SourceFile,
}

/// Index of a module within a [`Package`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ModuleId(pub usize);

/// One module: the root file, a module file, or an inline `mod name { }`.
#[derive(Debug, Clone)]
pub struct ModuleData {
    /// Module name; empty for the root module.
    pub name: String,
    /// Enclosing module.
    pub parent: Option<ModuleId>,
    /// Names from the root, excluding the root itself.
    pub path: Vec<String>,
    /// File holding the module's items.
    pub file: FileId,
    /// Region of `file` the module covers.
    pub span: Span,
    /// Location of the declaring `mod` item's name.
    pub decl: Option<(FileId, Span)>,
    /// Items declared directly in the module.
    pub items: Vec<Item>,
    /// Child modules by name.
    pub children: BTreeMap<String, ModuleId>,
}

impl ModuleData {
    /// `crate::a::b` style display path.
    pub fn display_path(&self) -> String {
        std::iter::once("crate")
            .chain(self.path.iter().map(String::as_str))
            .collect::<Vec<_>>()
            .join("::")
    }
}

/// A loaded, parsed package before semantic analysis.
#[derive(Debug, Clone)]
pub struct Package {
    /// Root file path.
    pub root: PathBuf,
    /// Every loaded file; `FileId(0)` is the root.
    pub files: Vec<SourceData>,
    /// Every module; `ModuleId(0)` is the root.
    pub modules: Vec<ModuleData>,
    /// Syntax and module-loading diagnostics.
    pub diagnostics: Vec<Diagnostic>,
}

impl Package {
    /// Find a loaded file by path.
    pub fn file_id(&self, path: &Path) -> Option<FileId> {
        self.files
            .iter()
            .position(|file| file.path == path)
            .map(FileId)
    }

    /// The innermost module covering `offset` in `file`.
    pub fn module_at(&self, file: FileId, offset: usize) -> Option<ModuleId> {
        self.modules
            .iter()
            .enumerate()
            .filter(|(_, module)| module.file == file && module.span.contains(offset))
            .min_by_key(|(_, module)| module.span.end - module.span.start)
            .map(|(index, _)| ModuleId(index))
    }

    /// Module data by id.
    pub fn module(&self, id: ModuleId) -> &ModuleData {
        &self.modules[id.0]
    }
}

/// Find the package root for a source file: the nearest ancestor directory
/// containing [`ROOT_FILE`], or the file itself when there is none.
pub fn find_root(file: &Path, files: &dyn FileProvider) -> PathBuf {
    for dir in file.ancestors().skip(1) {
        let candidate = dir.join(ROOT_FILE);
        if candidate == file || files.read(&candidate).is_some() {
            return candidate;
        }
    }
    file.to_path_buf()
}

/// Load and parse the package rooted at `root`.
pub fn load(root: &Path, files: &dyn FileProvider) -> Package {
    let mut loader = Loader {
        provider: files,
        package: Package {
            root: root.to_path_buf(),
            files: Vec::new(),
            modules: Vec::new(),
            diagnostics: Vec::new(),
        },
    };
    let text = files.read(root).unwrap_or_default();
    let file = loader.add_file(root.to_path_buf(), text);
    if loader.package.files[0].text.is_empty() && files.read(root).is_none() {
        loader.package.diagnostics.push(Diagnostic::error(
            "io",
            file,
            Span::default(),
            format!("cannot read package root {}", root.display()),
        ));
    }
    let source = &loader.package.files[file.0];
    let items = source.ast.items.clone();
    let span = Span::new(0, source.text.len());
    let dir = root.parent().map(Path::to_path_buf).unwrap_or_default();
    loader.add_module(
        String::new(),
        None,
        Vec::new(),
        file,
        span,
        None,
        items,
        dir,
    );
    loader.package
}

struct Loader<'a> {
    provider: &'a dyn FileProvider,
    package: Package,
}

impl Loader<'_> {
    fn add_file(&mut self, path: PathBuf, text: String) -> FileId {
        let id = FileId(self.package.files.len());
        let (ast, errors) = parser::parse(&text);
        self.package.diagnostics.extend(
            errors
                .into_iter()
                .map(|error| Diagnostic::error("syntax", id, error.span, error.message)),
        );
        self.package.files.push(SourceData {
            lines: LineIndex::new(&text),
            path,
            text,
            ast,
        });
        id
    }

    #[allow(clippy::too_many_arguments)]
    fn add_module(
        &mut self,
        name: String,
        parent: Option<ModuleId>,
        path: Vec<String>,
        file: FileId,
        span: Span,
        decl: Option<(FileId, Span)>,
        items: Vec<Item>,
        child_dir: PathBuf,
    ) -> ModuleId {
        let id = ModuleId(self.package.modules.len());
        self.package.modules.push(ModuleData {
            name,
            parent,
            path: path.clone(),
            file,
            span,
            decl,
            items: items.clone(),
            children: BTreeMap::new(),
        });
        for item in &items {
            let ItemKind::Mod(module) = &item.kind else {
                continue;
            };
            let child_name = module.name.name.clone();
            if self.package.modules[id.0]
                .children
                .contains_key(&child_name)
            {
                // Reported as a duplicate definition during analysis.
                continue;
            }
            let mut child_path = path.clone();
            child_path.push(child_name.clone());
            let decl = Some((file, module.name.span));
            let child = if let Some(inline) = &module.inline {
                self.add_module(
                    child_name.clone(),
                    Some(id),
                    child_path,
                    file,
                    item.span,
                    decl,
                    inline.clone(),
                    child_dir.join(&child_name),
                )
            } else {
                let flat = child_dir.join(format!("{child_name}.sand"));
                let nested = child_dir.join(&child_name).join("mod.sand");
                let found = [(flat, false), (nested, true)].into_iter().find_map(
                    |(candidate, owns_dir)| {
                        self.provider
                            .read(&candidate)
                            .map(|text| (candidate, text, owns_dir))
                    },
                );
                let Some((candidate, text, _)) = found else {
                    self.package.diagnostics.push(
                        Diagnostic::error(
                            "module",
                            file,
                            module.name.span,
                            format!("file not found for module `{child_name}`"),
                        )
                        .with_help(format!(
                            "create `{child_name}.sand` or `{child_name}/mod.sand` in {}",
                            child_dir.display()
                        )),
                    );
                    continue;
                };
                if let Some(existing) = self.package.file_id(&candidate) {
                    self.package.diagnostics.push(Diagnostic::error(
                        "module",
                        file,
                        module.name.span,
                        format!(
                            "{} is already loaded as another module",
                            self.package.files[existing.0].path.display()
                        ),
                    ));
                    continue;
                }
                let child_file = self.add_file(candidate, text);
                let source = &self.package.files[child_file.0];
                let items = source.ast.items.clone();
                let span = Span::new(0, source.text.len());
                // Both `combat.sand` and `combat/mod.sand` keep children in `combat/`.
                self.add_module(
                    child_name.clone(),
                    Some(id),
                    child_path,
                    child_file,
                    span,
                    decl,
                    items,
                    child_dir.join(&child_name),
                )
            };
            self.package.modules[id.0]
                .children
                .insert(child_name, child);
        }
        id
    }
}
