use sand::api;

#[api(
    path = "sand::testing::Status",
    aliases = ["sand::prelude::Status"],
    summary = "Represents whether a generated resource is ready.",
    context = "The status lets author code distinguish resources that can be emitted from resources still awaiting configuration.",
    minecraft = "Ready resources may be written into the datapack; pending resources are not emitted.",
    use_when = ["Branching on resource readiness"],
    avoid_when = ["Representing mutable Minecraft runtime state"],
    example = "Status::Ready",
    variants(
        Ready = "Marks a resource whose required authoring data is complete.",
        Pending = "Marks a resource that still requires author configuration."
    )
)]
pub enum Status {
    Ready,
    Pending,
}

#[api(
    path = "sand::testing::VersionFailure",
    summary = "Reports why a version request could not be resolved.",
    context = "The error keeps malformed input distinct from syntactically valid but unknown releases.",
    minecraft = "Resolution determines the pack formats and Minecraft features Sand may target.",
    use_when = ["Reporting a version configuration error"],
    avoid_when = ["Representing an accepted target version"],
    example = "VersionFailure::Parse(\"bad\".to_owned())",
    variants(
        Parse = "Carries malformed version text.",
        Unknown = "Carries an unverified version and the recovery hint."
    ),
    variant_fields(
        Parse = ["The original malformed version text."],
        Unknown(requested = "The syntactically valid version that has no verified profile.", hint = "The recommended recovery action.")
    )
)]
pub enum VersionFailure {
    Parse(String),
    Unknown { requested: String, hint: String },
}

#[api(
    path = "sand::testing::ResourceInfo",
    summary = "Carries the stable identity and enabled state of a resource.",
    context = "Resource metadata is inspected by author tooling before datapack emission.",
    minecraft = "The identifier selects a datapack resource and enabled controls whether it is emitted.",
    use_when = ["Inspecting authored resource metadata"],
    avoid_when = ["Tracking per-player runtime state"],
    example = "ResourceInfo { id: 1, enabled: true }",
    fields(
        id = "Stores the stable numeric fixture identity.",
        enabled = "Records whether export should include this resource."
    )
)]
pub struct ResourceInfo {
    pub id: u32,
    pub enabled: bool,
    internal: bool,
}

impl ResourceInfo {
    #[api(
        kind = "associated_const",
        path = "sand::testing::ResourceInfo::DEFAULT_ENABLED",
        summary = "Defines whether new resource metadata starts enabled.",
        context = "The default keeps constructors and generated schemas aligned.",
        minecraft = "Enabled resources are eligible for datapack emission.",
        use_when = ["Implementing a resource constructor"],
        avoid_when = ["Reading an existing resource's enabled state"],
        example = "ResourceInfo::DEFAULT_ENABLED",
    )]
    pub const DEFAULT_ENABLED: bool = true;
}

pub trait ResourceSchema {
    #[api(
        path = "sand::testing::ResourceSchema::Id",
        summary = "Names the typed identifier used by a resource schema.",
        context = "Schemas retain their identifier type across builder and export stages.",
        minecraft = "The identifier ultimately selects the resource's namespaced datapack path.",
        use_when = ["Implementing a typed resource schema"],
        avoid_when = ["A raw unvalidated string is being passed through"],
        example = "type Id = u32;",
    )]
    type Id;

    #[api(
        path = "sand::testing::ResourceSchema::SUPPORTED",
        summary = "Reports whether this schema can be emitted by the current implementation.",
        context = "Implementations use the constant to advertise export support without constructing a value.",
        minecraft = "Unsupported schemas produce no datapack resource.",
        use_when = ["Inspecting a schema implementation"],
        avoid_when = ["Checking a particular resource instance"],
        example = "Schema::SUPPORTED",
    )]
    const SUPPORTED: bool;
}

fn main() {
    let value = ResourceInfo { id: 1, enabled: true, internal: false };
    assert!(value.enabled && !value.internal);
}
