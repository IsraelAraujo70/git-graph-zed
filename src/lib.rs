mod git;

use git::{GraphCollector, SlashCommandOptions};
use zed_extension_api as zed;

const GIT_GRAPH_COMMAND: &str = "git-graph";

struct GitGraphExtension;

impl zed::Extension for GitGraphExtension {
    fn new() -> Self
    where
        Self: Sized,
    {
        GitGraphExtension
    }

    fn run_slash_command(
        &self,
        command: zed::SlashCommand,
        args: Vec<String>,
        worktree: Option<&zed::Worktree>,
    ) -> zed::Result<zed::SlashCommandOutput> {
        if command.name != GIT_GRAPH_COMMAND {
            return Err(format!("unknown slash command `{}`", command.name));
        }
        let worktree = worktree.ok_or_else(|| {
            "git graph slash command requires an attached workspace/root".to_string()
        })?;
        let options = SlashCommandOptions::from_args(&args).map_err(|err| err.to_string())?;
        let collector = GraphCollector::new(worktree).map_err(|err| err.to_string())?;
        let graph = collector
            .collect_graph(options.limit)
            .map_err(|err| err.to_string())?;
        let text = zed::serde_json::to_string_pretty(&graph)
            .map_err(|err| format!("failed to serialize graph: {err}"))?;
        Ok(zed::SlashCommandOutput {
            text,
            sections: Vec::new(),
        })
    }
}

zed::register_extension!(GitGraphExtension);
