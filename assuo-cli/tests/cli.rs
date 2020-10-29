use assert_cmd::Command;
use predicates::prelude::*;

// EXPECTATION FOR ARGUMENT PARSING:
// assuo aims to "do one thing, and do it right". our arg parsing aims to capture the unix philosophy by giving a
// similar experience to what tools like `cat` offer.
//
// therefore, all we support is printing to stdout and reading from stdin.
//
// OFFICIALLY SANCTIONED & SUPPORTED SCENARIOS:
//
//     print out the help
// assuo --help
// assuo -h
// assuo /?
//
//     prints out a template assuo.toml
// assuo --init
// assuo -i
//
//     run patches for an assuo file named `assuo.toml`
// cat assuo.toml | assuo
//
//     run patches for an assuo file named `x`
// cat x | assuo
//
//     run patches for an assuo file located at the URL `https://x`
// wget -O - https://x | assuo

fn cmd() -> Result<Command, assert_cmd::cargo::CargoError> {
    Command::cargo_bin("assuo")
}

// == tests ==

// help

#[test]
fn when_help_arg_specified_help_is_printed() -> Result<(), Box<dyn std::error::Error>> {
    cmd()?
        .arg("--help")
        .assert()
        .success()
        .stderr(predicate::str::contains("USAGE"));

    cmd()?
        .arg("-h")
        .assert()
        .success()
        .stderr(predicate::str::contains("USAGE"));

    cmd()?
        .arg("/?")
        .assert()
        .success()
        .stderr(predicate::str::contains("USAGE"));

    Ok(())
}

#[test]
fn init_prints_valid_assuo_toml() -> Result<(), Box<dyn std::error::Error>> {
    cmd()?
        .arg("--init")
        .assert()
        .success()
        .stdout(predicates::function::function(|bytes: &[u8]| {
            std::str::from_utf8(bytes)
                .and_then(|payload| {
                    if let Ok(_) = assuo::models::try_parse(payload) {
                        Ok(true)
                    } else {
                        Ok(false)
                    }
                })
                .unwrap_or(false)
        }));

    Ok(())
}

#[test]
fn when_stdin_is_supplied_it_is_parsed() -> Result<(), Box<dyn std::error::Error>> {
    cmd()?
        .write_stdin(
            r#"
[source]
text = "stuff being piped from stdin!"
"#,
        )
        .assert()
        .success()
        .stdout(predicate::eq("stuff being piped from stdin!"));

    Ok(())
}
