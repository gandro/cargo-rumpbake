extern crate cargo;
extern crate rustc_serialize;

use std::error::Error;

use cargo::ops;
use cargo::core::{Source};
use cargo::sources::path::PathSource;
use cargo::util::{CliResult, CliError, Config};
use cargo::util::important_paths::{find_root_manifest_for_cwd};

#[derive(RustcDecodable)]
struct Options {
    flag_output: Option<String>,
    flag_config: Option<String>,
    flag_bins: Vec<String>,
    flag_examples: Vec<String>,
    flag_jobs: Option<u32>,
    flag_features: Vec<String>,
    flag_no_default_features: bool,
    flag_target: Option<String>,
    flag_manifest_path: Option<String>,
    flag_verbose: bool,
    flag_quiet: bool,
    flag_color: Option<String>,
    flag_release: bool,
    arg_config: String,
}

pub const USAGE: &'static str = "
Builds and bakes your binaries into a rumprun image

Usage:
    cargo rumpbake [options] <config>
    cargo rumpbake -h | --help

Rumpbake options:
    -c PATH, --config PATH  Rumpbake config file
    -o NAME, --output NAME  Name of the generated image (default <crate>.img)

Cargo options:
    -h, --help              Print this message
    --bin NAME              Name of the bin target to build and bake
    --example NAME          Name of the example target to build and bake
    -j N, --jobs N          The number of jobs to run in parallel
    --release               Build artifacts in release mode, with optimizations
    --features FEATURES     Space-separated list of features to also build
    --no-default-features   Do not build the `default` feature
    --target TRIPLE         Build for the target triple
    --manifest-path PATH    Path to the manifest to execute
    -v, --verbose           Use verbose output
    -q, --quiet             No output printed to stdout
    --color WHEN            Coloring: auto, always, never

Builds a binary target for rumprun and then bakes it into an rumprun unikernel
image, using the specified rumpbake <config>, for example 'hw_generic'.
";

fn main() {
    cargo::execute_main_without_stdin(execute, false, USAGE);
}

fn execute(options: Options, config: &Config) -> CliResult<Option<()>> {
    try!(config.shell().set_verbosity(options.flag_verbose, options.flag_quiet));
    try!(config.shell().set_color_config(options.flag_color.as_ref().map(|s| &s[..])));

    let root = try!(find_root_manifest_for_cwd(options.flag_manifest_path));
    let mut src = try!(PathSource::for_path(&root.parent().unwrap(), config));
    try!(src.update());
    let pkg = try!(src.root_package());

    // FIXME(gandro): should autodetect target fallback 
    let target = options.flag_target.as_ref()
                        .map(|t| &t[..])
                        .or(Some("x86_64-rumprun-netbsd"));

    let filter = if options.flag_examples.is_empty() && options.flag_bins.is_empty() {
        ops::CompileFilter::Everything
    } else {
        ops::CompileFilter::Only {
            lib: false, tests: &[], benches: &[],
            bins: &options.flag_bins,
            examples: &options.flag_examples,
        }
    };

    let compile_opts = ops::CompileOptions {
        config: config,
        jobs: options.flag_jobs,
        target: target,
        features: &options.flag_features,
        no_default_features: options.flag_no_default_features,
        spec: None,
        exec_engine: None,
        release: options.flag_release,
        mode: ops::CompileMode::Build,
        filter: filter,
        target_rustc_args: None,
    };

    let compile = try!(ops::compile(&root, &compile_opts));
    
    if compile.binaries.is_empty() {
        return Err(CliError::new(
                    "a bin target must be available for `cargo rumpbake`", 1));
    }
    
    let output = options.flag_output.unwrap_or({
        format!("{}.img", pkg.name())
    });

    try!(config.shell().status("Baking", &output)
                       .map_err(|err| CliError::new(err.description(), 1)));

    let mut rumpbake = try!(cargo::util::process("rumpbake"));
    options.flag_config.map(|config| rumpbake.arg("-c").arg(config));
    rumpbake
        .arg(&options.arg_config)
        .arg(&output)
        .args(&compile.binaries)
        .env("RUMPRUN_WARNING_STFU", "please");
    
    try!(config.shell().verbose(|c| c.status("Running", &rumpbake))
                       .map_err(|err| CliError::new(err.description(), 1)));
    
    try!(rumpbake.exec_with_output()
                .map_err(|err| CliError::from_error(err, 1)));

    Ok(None)
}
