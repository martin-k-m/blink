/// A single dependency entry as declared in a manifest, before any
/// project-level interpretation (framework detection, etc.) is applied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawDependency {
    pub name: String,
    pub version: String,
    pub dev: bool,
}
