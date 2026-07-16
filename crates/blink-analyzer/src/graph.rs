use blink_core::Project;

/// A single node in a [`DependencyGraph`]: one declared dependency.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DependencyNode {
    pub name: String,
    pub version: String,
    pub dev: bool,
}

/// A shallow dependency graph rooted at the scanned project.
///
/// Blink builds this from the project's own manifest (direct dependencies
/// only) rather than fetching a full transitive tree from a registry, so it
/// stays fast, offline, and deterministic.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DependencyGraph {
    pub root: String,
    pub nodes: Vec<DependencyNode>,
}

impl DependencyGraph {
    pub fn from_project(project: &Project) -> Self {
        let nodes = project
            .dependencies
            .iter()
            .map(|dep| DependencyNode {
                name: dep.name.clone(),
                version: dep.version.clone(),
                dev: dep.dev,
            })
            .collect();

        Self {
            root: project.name.clone(),
            nodes,
        }
    }

    pub fn direct_count(&self) -> usize {
        self.nodes.len()
    }

    /// Render the graph as an indented ASCII tree, e.g.:
    ///
    /// ```text
    /// my-app
    /// ├── react
    /// └── react-dom
    /// ```
    pub fn render_tree(&self) -> String {
        let mut out = String::new();
        out.push_str(&self.root);
        out.push('\n');

        let count = self.nodes.len();
        for (i, node) in self.nodes.iter().enumerate() {
            let is_last = i + 1 == count;
            let branch = if is_last { "└── " } else { "├── " };
            out.push_str(branch);
            out.push_str(&node.name);
            if node.dev {
                out.push_str(" (dev)");
            }
            out.push('\n');
        }

        out
    }
}
