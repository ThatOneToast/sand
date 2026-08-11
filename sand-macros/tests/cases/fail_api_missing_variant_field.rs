use sand::api;

#[api(
    summary = "Reports why a target release could not be resolved.",
    context = "The error distinguishes malformed text from an unknown but valid release.",
    minecraft = "Version resolution selects Minecraft pack formats and available features.",
    use_when = ["Reporting target-version configuration failures"],
    avoid_when = ["Representing an accepted target release"],
    example = "VersionFailure::Parse(\"bad\".into())",
    variants(Parse = "Carries malformed source text.", Unknown = "Carries an unverified release."),
    variant_fields(Parse = ["The malformed version text."])
)]
pub enum VersionFailure {
    Parse(String),
    Unknown { requested: String, hint: String },
}

fn main() {}
