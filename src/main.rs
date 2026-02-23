use std::path::Path;

use rlox::error::Error;

fn main() -> rlox::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    match args.as_slice() {
        [_] => rlox_tree_walk::run_prompt()?,
        [_, flag, script] if flag == "--vm" => rlox_vm::run_file(Path::new(script))?,
        [_, script] => rlox_tree_walk::run_file(Path::new(script))?,
        _ => return Err(Error::Cli),
    };
    Ok(())
}
