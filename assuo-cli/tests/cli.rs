use assert_cmd::Command;
use predicates::prelude::*;

// ASSUO CLI:
//
//     print out the help
// assuo --help
// assuo -h
// assuo /?
//
//     prints out a template assuo.toml (so it can be piped into a file)
// assuo --init
// assuo -i
//
//     run patches for an assuo file named `assuo.toml`
// cat assuo.toml | assuo
// type assuo.toml | assuo
//
//     run patches for an assuo file located at the URL `https://x`
// wget -O - https://x | assuo

fn cmd() -> Result<Command, assert_cmd::cargo::CargoError> {
    Command::cargo_bin("assuo")
}

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
        .stdout(predicates::function::function(|bytes| {
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
