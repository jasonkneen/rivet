use anyhow::*;
use clap::Subcommand;

/// Commands for managing projects
#[derive(Subcommand)]
pub enum SubCommand {
	/// Open the project dashboard in a browser
	#[clap(alias = "v")]
	View,
}

impl SubCommand {
	pub async fn execute(&self) -> Result<()> {
		match &self {
			SubCommand::View => {
				let ctx = crate::util::login::load_or_login().await?;

				let url = format!(
					"{hub}/projects/{proj}",
					hub = ctx.bootstrap.origins.hub,
					proj = ctx.project.name_id,
				);

				webbrowser::open_browser_with_options(
					webbrowser::Browser::Default,
					&url,
					webbrowser::BrowserOptions::new().with_suppress_output(true),
				)?;

				Ok(())
			}
		}
	}
}
