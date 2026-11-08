//! The agent skill's tool reference must list every MCP tool, so the skill
//! cannot go stale when a tool is added.

use std::fs;
use std::path::PathBuf;

fn tool_names(path: PathBuf) -> Vec<String> {
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let tools: serde_json::Value = serde_json::from_str(&text).expect("tool file is JSON");
    tools
        .as_array()
        .expect("tool file is an array")
        .iter()
        .map(|t| t["name"].as_str().expect("tool has a name").to_owned())
        .collect()
}

#[test]
fn skill_reference_lists_every_tool() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let reference = fs::read_to_string(manifest.join("../../skills/kanso/references/tools.md"))
        .expect("skills/kanso/references/tools.md exists");
    let mut missing = Vec::new();
    let mut count = 0;
    for file in ["tools.json", "kanso_tools.json", "task_tools.json"] {
        for name in tool_names(manifest.join("src/mcp").join(file)) {
            count += 1;
            if !reference.contains(&format!("### `{name}`")) {
                missing.push(format!("{name} ({file})"));
            }
        }
    }
    assert!(count > 0, "no tools found");
    assert!(
        missing.is_empty(),
        "skills/kanso/references/tools.md is missing: {}. Add a `### `name`` section for each.",
        missing.join(", ")
    );
}

#[test]
fn skill_has_install_frontmatter() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let skill = fs::read_to_string(manifest.join("../../skills/kanso/SKILL.md"))
        .expect("skills/kanso/SKILL.md exists");
    assert!(skill.starts_with("---\nname: kanso\ndescription: "));
}
