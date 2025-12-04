use std::path::Path;

use rlox::error::Error;
use rlox_tree_walk::runtime;

fn main() -> rlox::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    match args.as_slice() {
        [_] => runtime::run_prompt()?,
        [_, script_path] => runtime::run_file(Path::new(script_path))?,
        _ => return Err(Error::Cli),
    };
    Ok(())
}
