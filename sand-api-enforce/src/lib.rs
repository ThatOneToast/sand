#![forbid(unsafe_code)]

//! Stable-Rust source auditing for public API contracts.
//!
//! An attribute macro can validate a contract that exists, but it cannot see
//! an unannotated sibling. Build scripts call this crate so ordinary
//! `cargo check` and `cargo build` reject new public items without `#[api]`.

use std::collections::BTreeSet;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use syn::spanned::Spanned;

mod reachable;
mod scope;

pub use reachable::{
    ContractIdentity, GeneratedApi, ReachabilityError, ReachableApi, ReachableKind, SourceCrate,
    SurfaceGraph, audit_reachable_surface,
};
pub use scope::{ApiScope, ScopeFailure, ScopeManifest, ScopeReport, ScopeReportEntry, ScopeState};

/// One source tree included in Sand's supported public surface.
#[derive(Clone, Debug)]
pub struct SurfaceRoot {
    /// Rust source file to inspect.
    pub source: PathBuf,
    /// Canonical facade module corresponding to the source file.
    pub canonical_module: String,
}

/// A missing-contract diagnostic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Violation {
    pub source: PathBuf,
    pub line: usize,
    pub canonical_path: String,
    pub item_kind: &'static str,
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}: public API `{}` ({}) is missing #[api]",
            self.source.display(),
            self.line,
            self.canonical_path,
            self.item_kind
        )
    }
}

/// Inspect every configured root and return all violations in deterministic
/// source/path order.
pub fn audit(roots: &[SurfaceRoot]) -> Result<(), Vec<Violation>> {
    audit_with_contracts(roots, &BTreeSet::new())
}

/// Inspect roots while accepting exact canonical identities supplied by a
/// generated or facade-owned contract table. This is used for re-exports,
/// whose defining item cannot carry Sand's procedural attribute.
pub fn audit_with_contracts(
    roots: &[SurfaceRoot],
    contracted_paths: &BTreeSet<String>,
) -> Result<(), Vec<Violation>> {
    let mut violations = Vec::new();
    for root in roots {
        audit_file(root, contracted_paths, &mut violations);
    }
    violations.sort_by(|left, right| {
        left.source
            .cmp(&right.source)
            .then_with(|| left.line.cmp(&right.line))
            .then_with(|| left.canonical_path.cmp(&right.canonical_path))
    });
    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
    }
}

/// Build-script entry point. Panics with stable, actionable diagnostics so
/// Cargo stops the normal compilation path.
pub fn enforce(roots: &[SurfaceRoot]) {
    if let Err(violations) = audit(roots) {
        let rendered = violations
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        panic!("Sand public API contract enforcement failed:\n{rendered}");
    }
}

/// Build-script entry point with exact identities supplied by generated or
/// facade-owned contract metadata.
pub fn enforce_with_contracts(roots: &[SurfaceRoot], contracted_paths: &BTreeSet<String>) {
    if let Err(violations) = audit_with_contracts(roots, contracted_paths) {
        let rendered = violations
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        panic!("Sand public API contract enforcement failed:\n{rendered}");
    }
}

fn audit_file(
    root: &SurfaceRoot,
    contracted_paths: &BTreeSet<String>,
    violations: &mut Vec<Violation>,
) {
    let source = fs::read_to_string(&root.source)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", root.source.display()));
    let parsed = syn::parse_file(&source)
        .unwrap_or_else(|error| panic!("failed to parse {}: {error}", root.source.display()));
    inspect_items(
        &parsed.items,
        &root.source,
        &root.canonical_module,
        false,
        contracted_paths,
        violations,
    );
}

fn inspect_items(
    items: &[syn::Item],
    source: &Path,
    module: &str,
    excluded_parent: bool,
    contracted_paths: &BTreeSet<String>,
    violations: &mut Vec<Violation>,
) {
    for item in items {
        let attrs = item_attrs(item);
        let excluded = excluded_parent || doc_hidden(attrs) || module.ends_with("::__private");
        match item {
            syn::Item::Mod(value) => {
                let path = format!("{module}::{}", value.ident);
                check_public(
                    &value.vis,
                    attrs,
                    source,
                    item.span(),
                    &path,
                    "module",
                    excluded,
                    contracted_paths,
                    violations,
                );
                if let Some((_, nested)) = &value.content {
                    inspect_items(
                        nested,
                        source,
                        &path,
                        excluded,
                        contracted_paths,
                        violations,
                    );
                }
            }
            syn::Item::Impl(value) => {
                let owner = compact_tokens(&value.self_ty);
                for child in &value.items {
                    match child {
                        syn::ImplItem::Fn(method) => check_public(
                            &method.vis,
                            &method.attrs,
                            source,
                            method.span(),
                            &format!("{module}::{owner}::{}", method.sig.ident),
                            "method",
                            excluded || doc_hidden(&method.attrs),
                            contracted_paths,
                            violations,
                        ),
                        syn::ImplItem::Const(constant) => check_public(
                            &constant.vis,
                            &constant.attrs,
                            source,
                            constant.span(),
                            &format!("{module}::{owner}::{}", constant.ident),
                            "associated constant",
                            excluded || doc_hidden(&constant.attrs),
                            contracted_paths,
                            violations,
                        ),
                        syn::ImplItem::Type(ty) => check_public(
                            &ty.vis,
                            &ty.attrs,
                            source,
                            ty.span(),
                            &format!("{module}::{owner}::{}", ty.ident),
                            "associated type",
                            excluded || doc_hidden(&ty.attrs),
                            contracted_paths,
                            violations,
                        ),
                        _ => {}
                    }
                }
            }
            syn::Item::Trait(value) => {
                let path = format!("{module}::{}", value.ident);
                check_public(
                    &value.vis,
                    attrs,
                    source,
                    item.span(),
                    &path,
                    "trait",
                    excluded,
                    contracted_paths,
                    violations,
                );
                if is_public(&value.vis) && !excluded {
                    for child in &value.items {
                        let (child_attrs, name, kind, span) = match child {
                            syn::TraitItem::Fn(method) => (
                                method.attrs.as_slice(),
                                method.sig.ident.to_string(),
                                "trait method",
                                method.span(),
                            ),
                            syn::TraitItem::Const(constant) => (
                                constant.attrs.as_slice(),
                                constant.ident.to_string(),
                                "trait constant",
                                constant.span(),
                            ),
                            syn::TraitItem::Type(ty) => (
                                ty.attrs.as_slice(),
                                ty.ident.to_string(),
                                "trait associated type",
                                ty.span(),
                            ),
                            _ => continue,
                        };
                        let child_path = format!("{path}::{name}");
                        if !has_api(child_attrs)
                            && !doc_hidden(child_attrs)
                            && !contracted_paths.contains(&child_path)
                        {
                            push_violation(source, span, child_path, kind, violations);
                        }
                    }
                }
            }
            syn::Item::Macro(value) => {
                if !excluded
                    && value
                        .attrs
                        .iter()
                        .any(|attr| attr.path().is_ident("macro_export"))
                    && !has_api(&value.attrs)
                {
                    let name = value
                        .ident
                        .as_ref()
                        .map_or_else(|| "<anonymous>".to_owned(), ToString::to_string);
                    push_violation(
                        source,
                        value.span(),
                        format!("{module}::{name}"),
                        "macro",
                        violations,
                    );
                }
            }
            _ => {
                if let Some((vis, name, kind)) = public_identity(item) {
                    check_public(
                        vis,
                        attrs,
                        source,
                        item.span(),
                        &format!("{module}::{name}"),
                        kind,
                        excluded,
                        contracted_paths,
                        violations,
                    );
                }
            }
        }
    }
}

fn public_identity(item: &syn::Item) -> Option<(&syn::Visibility, String, &'static str)> {
    match item {
        syn::Item::Const(value) => Some((&value.vis, value.ident.to_string(), "constant")),
        syn::Item::Enum(value) => Some((&value.vis, value.ident.to_string(), "enum")),
        syn::Item::Fn(value) => Some((&value.vis, value.sig.ident.to_string(), "function")),
        syn::Item::Static(value) => Some((&value.vis, value.ident.to_string(), "static")),
        syn::Item::Struct(value) => Some((&value.vis, value.ident.to_string(), "struct")),
        syn::Item::Type(value) => Some((&value.vis, value.ident.to_string(), "type alias")),
        syn::Item::Union(value) => Some((&value.vis, value.ident.to_string(), "union")),
        syn::Item::Use(value) => Some((&value.vis, compact_tokens(&value.tree), "re-export")),
        _ => None,
    }
}

#[allow(clippy::too_many_arguments)]
fn check_public(
    visibility: &syn::Visibility,
    attrs: &[syn::Attribute],
    source: &Path,
    span: proc_macro2::Span,
    canonical_path: &str,
    item_kind: &'static str,
    excluded: bool,
    contracted_paths: &BTreeSet<String>,
    violations: &mut Vec<Violation>,
) {
    if is_public(visibility)
        && !excluded
        && !has_api(attrs)
        && !contracted_paths.contains(canonical_path)
    {
        push_violation(
            source,
            span,
            canonical_path.to_owned(),
            item_kind,
            violations,
        );
    }
}

fn push_violation(
    source: &Path,
    span: proc_macro2::Span,
    canonical_path: String,
    item_kind: &'static str,
    violations: &mut Vec<Violation>,
) {
    violations.push(Violation {
        source: source.to_path_buf(),
        line: span.start().line,
        canonical_path,
        item_kind,
    });
}

fn item_attrs(item: &syn::Item) -> &[syn::Attribute] {
    match item {
        syn::Item::Const(value) => &value.attrs,
        syn::Item::Enum(value) => &value.attrs,
        syn::Item::ExternCrate(value) => &value.attrs,
        syn::Item::Fn(value) => &value.attrs,
        syn::Item::ForeignMod(value) => &value.attrs,
        syn::Item::Impl(value) => &value.attrs,
        syn::Item::Macro(value) => &value.attrs,
        syn::Item::Mod(value) => &value.attrs,
        syn::Item::Static(value) => &value.attrs,
        syn::Item::Struct(value) => &value.attrs,
        syn::Item::Trait(value) => &value.attrs,
        syn::Item::TraitAlias(value) => &value.attrs,
        syn::Item::Type(value) => &value.attrs,
        syn::Item::Union(value) => &value.attrs,
        syn::Item::Use(value) => &value.attrs,
        _ => &[],
    }
}

fn has_api(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        let path = attr.path();
        path.is_ident("api")
            || path
                .segments
                .last()
                .is_some_and(|segment| segment.ident == "api")
    })
}

fn doc_hidden(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("doc")
            && attr
                .parse_args::<syn::Ident>()
                .is_ok_and(|ident| ident == "hidden")
    })
}

fn is_public(visibility: &syn::Visibility) -> bool {
    matches!(visibility, syn::Visibility::Public(_))
}

fn compact_tokens(tokens: &impl quote::ToTokens) -> String {
    tokens.to_token_stream().to_string().replace(' ', "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catches_unannotated_functions_methods_and_trait_items() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("lib.rs");
        fs::write(
            &source,
            r#"
                pub fn forgotten(value: u32) -> u32 { value }
                pub struct Builder;
                impl Builder { pub fn build(self) {} }
                pub trait Extension { fn extend(&self); }
            "#,
        )
        .unwrap();
        let violations = audit(&[SurfaceRoot {
            source,
            canonical_module: "sand::fixture".into(),
        }])
        .unwrap_err();
        let paths = violations
            .iter()
            .map(|violation| violation.canonical_path.as_str())
            .collect::<Vec<_>>();
        assert!(paths.contains(&"sand::fixture::forgotten"));
        assert!(paths.contains(&"sand::fixture::Builder"));
        assert!(paths.contains(&"sand::fixture::Builder::build"));
        assert!(paths.contains(&"sand::fixture::Extension"));
        assert!(paths.contains(&"sand::fixture::Extension::extend"));
    }

    #[test]
    fn accepts_contracts_and_narrow_doc_hidden_exclusions() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("lib.rs");
        fs::write(
            &source,
            r#"
                #[api(summary = "complete")]
                pub fn supported() {}
                #[doc(hidden)]
                pub fn generated_wiring() {}
                pub(crate) fn internal() {}
            "#,
        )
        .unwrap();
        audit(&[SurfaceRoot {
            source,
            canonical_module: "sand::fixture".into(),
        }])
        .unwrap();
    }
}
