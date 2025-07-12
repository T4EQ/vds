use clap::Parser;
use xshell::cmd;

#[derive(Debug, clap::Args)]
struct BuildArgs {
    #[arg(long, short)]
    release: bool,
}

#[derive(Debug, clap::Args)]
struct RunArgs {
    #[command(flatten)]
    build_args: BuildArgs,
}

#[derive(Debug, clap::Subcommand)]
enum Command {
    Build(BuildArgs),
    Run(RunArgs),
}

#[derive(Debug, clap::Parser)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

fn build(args: &BuildArgs) -> anyhow::Result<()> {
    // TODO(javier-varez): Check and install build dependencies

    let release = args.release.then_some("--release");

    // First build the frontend and package it using trunk
    let shell = xshell::Shell::new()?;
    {
        let _dir = shell.push_dir("vds-site");
        cmd!(shell, "trunk build {release...}").run()?;
    }

    // Now build the backend
    cmd!(shell, "cargo build {release...}").run()?;

    Ok(())
}

fn run(args: &RunArgs) -> anyhow::Result<()> {
    build(&args.build_args)?;

    let release = args.build_args.release.then_some("--release");
    let shell = xshell::Shell::new()?;
    cmd!(
        shell,
        "cargo run {release...} --bin vds-server -- --web-data ./vds-site/dist"
    )
    .run()?;

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match &args.command {
        Command::Build(args) => build(args)?,
        Command::Run(args) => run(args)?,
    }

    Ok(())
}
