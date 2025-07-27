use clap::Parser;
use xshell::cmd;

#[derive(Debug, clap::Args)]
struct BuildArgs {
    #[arg(long, short)]
    release: bool,

    #[arg(long, short = 'j', default_value = "0")]
    num_threads: u64,

    #[arg(long)]
    offline: bool,

    #[arg(long)]
    target: Option<String>,
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
    let release = args.release.then_some("--release");
    let offline = args.offline.then_some("--offline");
    let num_threads = &(args.num_threads > 0).then_some(format!("-j={}", args.num_threads));
    let target = &args
        .target
        .as_ref()
        .map(|target| format!("--target={target}"));

    // First build the frontend and package it using trunk
    let shell = xshell::Shell::new()?;
    {
        let _dir = shell.push_dir("vds-site");
        cmd!(shell, "trunk build {offline...} {release...}").run()?;
    }

    // Now build the backend
    cmd!(
        shell,
        "cargo build {offline...} {release...} {num_threads...} {target...}"
    )
    .run()?;

    Ok(())
}

fn run(args: &RunArgs) -> anyhow::Result<()> {
    build(&args.build_args)?;

    let release = args.build_args.release.then_some("--release");
    let shell = xshell::Shell::new()?;
    cmd!(
        shell,
        "cargo run {release...} --bin vds-server -- --content-path video-data"
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
