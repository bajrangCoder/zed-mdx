use std::{env, fs};

use zed_extension_api::{self as zed, settings::LspSettings};

const SERVER_NAME: &str = "mdx-analyzer";
const SERVER_PATH: &str = "node_modules/.bin/mdx-language-server";
const PACKAGE_NAME: &str = "@mdx-js/language-server";

struct Mdx;

impl Mdx {
    fn server_exists(&self) -> bool {
        fs::metadata(SERVER_PATH).is_ok_and(|m| m.is_file())
    }

    fn server_script_path(
        &mut self,
        language_server_id: &zed_extension_api::LanguageServerId,
    ) -> zed::Result<String> {
        if !self.server_exists() {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::CheckingForUpdate,
            );
            let version = zed::npm_package_latest_version(PACKAGE_NAME)?;

            if zed::npm_package_installed_version(PACKAGE_NAME)?.as_ref() != Some(&version) {
                zed::set_language_server_installation_status(
                    language_server_id,
                    &zed::LanguageServerInstallationStatus::Downloading,
                );
                let result = zed::npm_install_package(PACKAGE_NAME, &version);
                if !self.server_exists() {
                    return result.and_then(|_| Err(format!("installed package '{PACKAGE_NAME}' did not contain expected path '{SERVER_PATH}'")));
                }
            }
        }

        Ok(SERVER_PATH.to_string())
    }
}

impl zed::Extension for Mdx {
    fn new() -> Self {
        Self
    }

    fn language_server_initialization_options(
        &mut self,
        _: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<Option<zed::serde_json::Value>> {
        let init_options = LspSettings::for_worktree(SERVER_NAME, worktree)
            .ok()
            .and_then(|settings| settings.initialization_options)
            .and_then(|options| options.get("typescript").cloned())
            .and_then(|options| options.as_object().cloned());

        let ts_enabled = init_options
            .as_ref()
            .and_then(|options| options.get("enabled").and_then(|enabled| enabled.as_bool()))
            .unwrap_or(true);
        let tsdk_path = init_options
            .as_ref()
            .and_then(|options| {
                options
                    .get("tsdk")
                    .and_then(|tsdk| tsdk.as_str())
                    .map(|s| s.to_owned())
                    .clone()
            })
            .unwrap_or(
                env::current_dir()
                    .unwrap()
                    .join(
                        "../../../languages/vtsls/node_modules/@vtsls/language-service/node_modules/typescript/lib",
                    )
                    .to_string_lossy()
                    .to_string(),
            );

        Ok(Some(zed::serde_json::json!({
            "typescript": {
                "enabled": ts_enabled,
                "tsdk": tsdk_path,
            },
        })))
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        _: &zed::Worktree,
    ) -> zed::Result<zed::Command> {
        let server_path = self.server_script_path(language_server_id)?;

        Ok(zed::Command {
            command: zed::node_binary_path()?,
            args: vec![
                env::current_dir()
                    .unwrap()
                    .join(&server_path)
                    .to_string_lossy()
                    .to_string(),
                "--stdio".to_owned(),
            ],
            env: Default::default(),
        })
    }
}

zed::register_extension!(Mdx);
