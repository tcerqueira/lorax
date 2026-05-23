use std::path::Path;

use rlox::error::Error;

fn main() -> rlox::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    match args.as_slice() {
        [_] => tree_walk::run_prompt()?,
        [_, flag] if flag == "--vm" => vm::run_prompt()?,
        [_, script] => tree_walk::run_file(Path::new(script))?,
        [_, flag, source] if flag == "-c" => tree_walk::run_source(source.clone())?,
        [_, flag, script] if flag == "--vm" => vm::run_file(Path::new(script))?,
        [_, vm_flag, c_flag, source] if vm_flag == "--vm" && c_flag == "-c" => {
            vm::run_source(source.clone())?
        }
        _ => return Err(Error::Cli),
    };
    Ok(())
}
