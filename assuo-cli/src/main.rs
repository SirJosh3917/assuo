use assuo::patch::do_patch;
use std::io::prelude::*;

#[paw::main]
// #[tokio::main(flavor = "current_thread")] 0.3+ only
fn main(args: paw::Args) -> Result<(), Box<dyn std::error::Error>> {
    for arg in args.skip(1) {
        if arg == "--init" || arg == "-i" {
            init();
            std::process::exit(0);
        }

        if arg == "--help" || arg == "-h" || arg == "/?" {
            help();
            std::process::exit(0);
        }
    }

    let mut buffer = Vec::new();
    std::io::stdin().lock().read_to_end(&mut buffer).unwrap();
    let assuo_config = String::from_utf8(buffer).unwrap();

    let config = assuo::models::try_parse(&assuo_config).unwrap();
    let patch = tokio::runtime::Runtime::new()?.block_on(do_patch(config))?;
    std::io::stdout().lock().write_all(&patch).unwrap();

    Ok(())
}

fn help() {
    eprintln!(
        "OVERVIEW: assuo patch maker

USAGE:
  assuo --init
  assuo --help
  cat assuo.toml | assuo

OPTIONS:
-h, --help   Prints help.
-i, --init   Makes a new blank assuo patch file."
    );
}

fn init() {
    let assuo_config = r#"[source]
text = "Hello!"

[[patch]]
do = "insert"
way = "post"
spot = 4
source = { text = ", World" }
"#;
    println!("{}", assuo_config);
}
