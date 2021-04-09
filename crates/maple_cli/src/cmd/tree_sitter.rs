use std::path::PathBuf;

use anyhow::{anyhow, Result};
use structopt::StructOpt;
use tree_sitter_tags::{TagsConfiguration, TagsContext};

/// Execute the shell command
#[derive(StructOpt, Debug, Clone)]
pub struct TreeSitter {
    /// Search term.
    #[structopt(index = 1, short, long)]
    pub word: String,

    /// Specify the working directory.
    #[structopt(long = "path", parse(from_os_str))]
    pub path: Option<PathBuf>,

    /// Definition kind.
    #[structopt(long = "kind")]
    pub kind: Option<String>,

    /// Specify the working directory.
    #[structopt(long = "cmd-dir", parse(from_os_str))]
    pub cmd_dir: Option<PathBuf>,
}

impl TreeSitter {
    pub fn run(&self) -> Result<()> {
        let mut context = TagsContext::new();

        let python_config = TagsConfiguration::new(
            tree_sitter_python::language(),
            tree_sitter_python::TAGGING_QUERY,
            "",
        )
        .unwrap();

        let javascript_config = TagsConfiguration::new(
            tree_sitter_javascript::language(),
            tree_sitter_javascript::TAGGING_QUERY,
            tree_sitter_javascript::LOCALS_QUERY,
        )
        .unwrap();

        let rust_config = TagsConfiguration::new(
            tree_sitter_rust::language(),
            tree_sitter_rust::TAGGING_QUERY,
            tree_sitter_rust::LOCALS_QUERY,
        )
        .unwrap();

        let source = if let Some(p) = self.path {
            std::fs::read(p)?.as_slice()
        } else {
            &b"class A { getB() { return c(); } }".to_vec()
        };

        let (tags_iter, root_node_has_error) = context
            .generate_tags(&javascript_config, source, None)
            .map_err(|e| anyhow!("tree sitter error: {:?}", e))?;
        let tags = tags_iter.filter_map(|x| x.ok()).collect::<Vec<_>>();

        for tag in tags.iter() {
            println!("tag: {:?}", tag);
            println!(
                "syntax_type_name: {}",
                python_config.syntax_type_name(tag.syntax_type_id)
            );
            // println!("kind: {:?}", tag.kind);
            // println!("range: {:?}", tag.range);
            // println!("name_range: {:?}", tag.name_range);
            // println!("docs: {:?}", tag.docs);
        }

        Ok(())
    }
}
