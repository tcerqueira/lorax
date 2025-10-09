use std::path::Path;

use rlox::error::Error;

fn main() -> rlox::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    match args.as_slice() {
        [_] => rlox::run_prompt(),
        [_, script_path] => rlox::run_file(Path::new(script_path)),
        _ => Err(Error::Cli),
    }
}
