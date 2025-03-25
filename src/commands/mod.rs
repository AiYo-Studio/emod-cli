use clap::{arg, Args, Parser, Subcommand};

pub mod components;
pub mod create;
pub mod release;

#[derive(Parser)]
#[command(
    name = "emod-cli",
    version = "1.0.0",
    about = "Convenient Management of NetEase Minecraft Mod Project",
    allow_external_subcommands = true,
    long_about = None,
    propagate_version = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Release a new version
    Release(ReleaseArgs),
    /// Create a new mod project
    Create(CreateArgs),
    /// Create a new component
    Components(ComponentsArgs),
}

#[derive(Args)]
pub struct ReleaseArgs {
    /// The path of the project
    #[arg(short, long)]
    pub path: Option<String>,
    /// The version of the project
    #[arg(short, long)]
    pub ver: Option<String>,
}

#[derive(Args)]
pub struct CreateArgs {
    /// The name of the mod
    #[arg(short, long)]
    pub name: String,
    /// Example target, default example is 'default'
    #[arg(short, long)]
    pub target: Option<String>,
}

#[derive(Args)]
pub struct ComponentsArgs {
    /// The path of the project
    #[arg(short, long)]
    pub path: Option<String>,
    /// The name of the component
    #[arg(short, long)]
    pub component: String,
    /// Import the path of the geo file.
    #[arg(short, long)]
    pub geo: Option<String>,
    /// Import the path of the texture file.
    #[arg(short, long)]
    pub texture: Option<String>,
    /// The item's identifier
    #[arg(short, long)]
    pub identifier: Option<String>
}
