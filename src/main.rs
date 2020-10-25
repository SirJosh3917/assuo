use atty::Stream;
use std::io::prelude::*;

pub mod models;
use models::*;

fn help() {
    println!(
        "OVERVIEW: assuo patch maker

USAGE: assuo [--url source_url]/[--file file_location]/[--init]/[--help]

OPTIONS:
--url    Loads an assuo patch file from the internet.
--file   Loads an assuo patch file from disk.
--init   Makes a new blank assuo patch file.
--help   Prints help.
"
    );
}

fn init(file_name: String) {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(file_name)
        .expect("couldn't open file for writing");

    file.write_all(
        r#"[source]
text = "Hello!"

[[patch]]
modify = "post insert"
spot = 5
source = { text = ", World" }
"#
        .as_bytes(),
    )
    .expect("couldn't write to file");
}

fn do_patch(mut file: AssuoFile) {
    // first, grab the source of the file
    eprintln!("resolving base source");
    let mut source = file.source.resolve();

    // TODO: resolve patches asynchronously here or something
    let mut patches = file.patch;

    // to apply patches, it would be weird and hacky to have to deal with
    // maintaining "okay add this much" after a certain index or whatever
    //
    // so we just re-order the patches so that we apply the last one first
    eprintln!("sorting patches by insertion order");
    patches.sort_by(|a, b| b.spot.partial_cmp(&a.spot).unwrap());

    // now, we can sequentially apply patches not worrying about the index
    for patch in patches {
        eprintln!("resolving patch");
        let bytes = patch.source.resolve();

        eprintln!("stitching to original source");
        for _ in source.splice(patch.spot..patch.spot, bytes) {}
    }

    std::io::stdout()
        .lock()
        .write(&source)
        .expect("to write to stdout");
}

#[paw::main]
fn main(args: paw::Args) {
    // ARGUMENT PARSING:
    // assuo aims to "do one thing, and do it right". our arg parsing aims to capture the unix philosophy by giving a
    // similar experience to what tools like `cat` offer.
    //
    // SUPPORTED SCENARIOS:
    //
    //     print out the help
    // assuo
    // assuo --help
    // assuo -h
    // assuo /help
    // assuo /h
    // assuo -?
    // assuo /?
    //
    //     initialize a blank assuo file named `assuo.toml`
    // assuo --init
    // assuo -i
    // assuo /init
    // assuo /i
    //
    //     initialize a blank assuo file named `x`
    // assuo --init x
    // assuo -i x
    // assuo /init x
    // assuo /i x
    //
    //     run patches for an assuo file named `assuo.toml`
    // assuo
    // cat assuo.toml | assuo
    //
    //     run patches for an assuo file named `x`
    // assuo x
    // assuo --file x
    // assuo -f x
    // assuo /file x
    // assuo /f x
    // cat x | assuo
    //
    //     run patches for an assuo file located at the URL `https://x`
    // assuo --url https://x
    // assuo -u https://x
    // assuo /url https://x
    // assuo /u https://x
    // wget -O - https://x | assuo

    let being_piped = !atty::is(atty::Stream::Stdin);
    let mut do_init = false;

    for arg in args.skip(1) {
        if do_init {
            init(arg);
            return;
        }

        let mut trim_for_arg = if arg.starts_with("--") {
            2
        } else if arg.starts_with("-") {
            1
        } else if arg.starts_with("/") {
            1
        } else {
            0
        };

        if trim_for_arg > 0 {
            let arg = &arg[trim_for_arg..];

            if arg == "?" || arg == "h" || arg == "help" {
                help();
                return;
            } else if arg == "i" || arg == "init" {
                do_init = true;
            }
        } else {
            let config =
                toml::from_str::<AssuoFile>(&std::fs::read_to_string(arg).unwrap()).unwrap();
            do_patch(config);
            return;
        }
    }

    if do_init {
        init(String::from("assuo.toml"));
        return;
    }

    // TODO: display help if no "assuo.toml" found (and print that no assuo.toml was found, showing help)
    let config =
        toml::from_str::<AssuoFile>(&std::fs::read_to_string("assuo.toml").unwrap()).unwrap();
    do_patch(config);
}
