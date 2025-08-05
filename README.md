# Offline Video Delivery System

[AID India](https://aidindia.in/) helps to bring education and educational resources to children in remote villages of India.
These villages often face challenges with the stability of their internet connections. In such an environment,
serving quality lectures can be challenging due to constraints of bandwidth, cost and latency.

The `Offline Video Delivery System` (`VDS`) is the result of a collaboration between AID India and T4EQ in an
attempt to bring quality educational videos to these children by caching the content locally at the
point where the data is served.

The `VDS` system consists of:
- A frontend video player
- A backend serving locally cached video content to devices in a local network.
- A management API to initiate the download of video content from remote servers,
as well as managing the local content available in the VDS.

The `VDS` functionality can run in low power devices such as raspberry pi and similar SBCs, and is
tailored such that it uses minimal resources during runtime.

## Getting started

Clone the repository with:

```sh
git clone --recurse-submodules git@github.com:T4EQ/vds.git
```

This will create a `vds` directory inside your current working directory.
The layout of this directory is as follows:

```
.
├── README.md           -> This file. Contains information about the project.
├── vds-server          -> The main web server application crate, build using actix-web.
├── vds-site            -> The frontend application crate, built using Yew. It is built as a wasm application and embedded into the vds-server binary.
│   ├── index.html      -> The entrypoint of the single-page web application. The body tag is populated from the Yew app.
│   ├── index.scss      -> The sylesheet applicable to the whole frontend.
│   └── ...             -> Other files belonging a regular Rust crate.
├── vds-api             -> A library crate containing data types that may be shared between frontend and backend.
├── xtask               -> A binary crate that implements tasks to build and run the whole project.
├── flake.nix           -> Declares the dependencies of the project in order to perform reproducible builds. You can safely ignore this file.
├── flake.lock          -> Pins the dependencies of the repository. You can safely ignore this file.
└── .cargo
   └── config.toml     -> Configuration file defining cargo aliases.
```

### Dependencies

This website relies on the following software:
- `Rust`, `v1.88`. It is recommended you install a toolchain using [rustup](rustup.rs), to help you manage multiple installs.
  Alternatively, using the `nix develop` environment will include the right versions out of the box.
- `trunk`, `v0.21` application bundler. See install instructions [here](https://trunkrs.dev/) 
- _Optional:_ `Nix` package manager. See install instructions [here](https://nixos.org/download/).

The `Nix` package manager is used to ensure that the build results are fully reproducible accross 
all machines. When using `Nix`, you don't need to worry about installing other tools separately. 
It is mostly intended for continuous integration/deployment in GitHub, so you may decide to skip 
its installation for your local development setup.

### Build with Cargo

To build the web server simply run:

```sh
# Build in debug mode:
cargo b

# Build in release mode:
cargo b
```

Note that `b` and `br` are cargo aliases defined in `.cargo/config.toml`. These aliases invoke `xtask` 
with the corresponding arguments to trigger a build. You can read more about the `xtask` pattern 
[here](https://github.com/matklad/cargo-xtask).

You will find the build under `target/{debug,release}/vds-server` depending on the build profile you used.
This is a self-contained binary that contains the frontend build inside itself.

Alternatively, you can serve the web server locally on port `8080` using:

```sh
# Run in debug mode:
cargo r

# Run in release mode:
cargo rr
```

Note that this command does not terminate, but keeps serving the website at `http://localhost:8080`. 

For development, you might want to use [`cargo-watch`](https://crates.io/crates/cargo-watch) to watch 
all file changes and automatically rebuild the `vds-server` (and `vds-site`) when they change, 
so that simply refreshing the web browser page displays the latest edits. 

```sh
cargo watch -x 'rr'
```

### Build with Nix

As mentioned above, this setup is mainly intended for continuous integration/deployment. However, 
if you have the `Nix` package manager installed in your system, it will automatically ensure you 
get the right tool versions out of the box. 

You can build the `vds-server` with:

```sh
nix build --extra-experimental-features 'nix-command flakes' .
```

And find the result under the `result/bin/vds-server` path.

Or you can enter a development shell with the full environment preconfigured and run the build steps 
listed in [Build with Cargo](#build-with-cargo), using:

```sh
nix develop --extra-experimental-features 'nix-command flakes' .
```

See [direnv](https://direnv.net/) to automatically execute `nix develop` when you change your working directory
to a subdirectory inside this repository.

Note that you do _not_ need `Nix` to use the right versions of the rust toolchain. Installing `Rust` 
via `rustup.rs` will make sure you get the right toolchain (indicated in the `rust-toolchain.toml`) 
file every time you run cargo.
