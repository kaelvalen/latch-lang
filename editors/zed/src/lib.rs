use zed_extension_api as zed;

struct LatchExtension;

impl zed::Extension for LatchExtension {
    fn new() -> Self {
        LatchExtension
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<zed::Command> {
        let path = worktree
            .which("latch")
            .unwrap_or_else(|| "latch".to_string());

        Ok(zed::Command {
            command: path,
            args: vec!["lsp".to_string()],
            env: Default::default(),
        })
    }
}

zed::register_extension!(LatchExtension);
