use assert_cmd::{assert::Assert, prelude::*};
use httptest::{matchers::request, responders::status_code, Expectation, Server};
use predicates::prelude::*;
use std::io::prelude::*;
use std::ops::{Deref, DerefMut};
use std::process::Stdio;
use tempfile::TempDir;

// == tests ==

// help

#[test]
fn help_mentions_all_commands() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cmd()?;

    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::eq(""))
        .stderr(predicate::str::contains("USAGE"))
        .stderr(predicate::str::contains("--help"))
        .stderr(predicate::str::contains("--init"))
        .stderr(predicate::str::contains("--url"))
        .stderr(predicate::str::contains("--file"));

    Ok(())
}

#[test]
fn when_no_args_and_no_assuo_toml_help_is_printed() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cmd()?;

    cmd.assert()
        .success()
        .stderr(predicate::str::contains("USAGE"));

    Ok(())
}

// init

#[test]
fn init_creates_assuo_toml_when_no_args_and_stdout_not_redirected(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cmd()?;

    cmd.arg("--init");
    cmd.assert()
        .success()
        .stdout(predicate::eq(""))
        .stderr(predicate::eq("Created 'assuo.toml'."));

    let config = std::fs::read_to_string("assuo.toml")?;
    assert!(config.contains("[source]"));

    Ok(())
}

#[test]
fn init_creates_file_when_arg_and_stdout_not_redirected() -> Result<(), Box<dyn std::error::Error>>
{
    let mut cmd = cmd()?;

    cmd.arg("--init").arg("custom_file");
    cmd.assert()
        .success()
        .stdout(predicate::eq(""))
        .stderr(predicate::eq("Created 'custom_file'."));

    let config = std::fs::read_to_string(cmd.workspace.as_ref().join("custom_file"))?;
    assert!(config.contains("[source]"));

    Ok(())
}

#[test]
fn init_prints_assuo_toml_when_no_args_and_stdout_redirected(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cmd()?;

    cmd.arg("--init").stdout(Stdio::piped());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("[source]"))
        .stderr(predicate::eq(""));

    Ok(())
}

// parsing file works related to input

#[test]
fn when_no_args_and_a_assuo_toml_exists_it_is_parsed() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cmd()?;

    std::fs::write(
        cmd.workspace.as_ref().join("assuo.toml"),
        r#"
[source]
text = "working"
"#,
    )?;

    cmd.assert()
        .success()
        .stdout(predicate::eq("working"))
        .stderr(predicate::str::contains(
            "assuo.toml exists - help not being printed.",
        ));

    Ok(())
}

#[test]
fn when_file_arg_has_file_it_is_parsed() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cmd()?;

    std::fs::write(
        cmd.workspace.as_ref().join("custom_file"),
        r#"
[source]
text = "working"
"#,
    )?;

    cmd.arg("--file").arg("custom_file");
    cmd.assert()
        .success()
        .stdout(predicate::eq("working"))
        .stderr(predicate::str::contains(
            "assuo.toml exists - help not being printed.",
        ));

    Ok(())
}

#[test]
fn when_url_arg_has_url_it_is_parsed() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cmd()?;

    let server = Server::run();
    server.expect(
        Expectation::matching(request::method_path("GET", "/target")).respond_with(
            status_code(200).body(
                r#"
[source]
text = "can read from site"
"#,
            ),
        ),
    );

    let target = server.url("/target");

    cmd.arg("--url").arg(target.to_string());
    cmd.assert()
        .success()
        .stdout(predicate::eq("can read from site"))
        .stderr(predicate::str::contains("/target")); // stderr should mention something about connecting to the site

    Ok(())
}

#[test]
fn when_stdin_is_supplied_it_is_parsed() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cmd()?;

    cmd.stdin(Stdio::piped());

    // workaround for it not being super easy to pipe data to stdin

    let mut output = cmd.spawn()?;
    let stdin = output.stdin.as_mut().unwrap();
    stdin.write_all(
        r#"
[source]
text = "stuff being piped from stdin!"
"#
        .as_bytes(),
    )?;
    let output = output.wait_with_output()?;
    let assert = Assert::new(output).append_context("command", format!("{:?}", cmd.deref()));

    assert
        .success()
        .stdout(predicate::eq("stuff being piped from stdin!"));

    Ok(())
}

#[test]
fn stdin_takes_priority() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = cmd()?;

    std::fs::write(
        cmd.workspace.as_ref().join("assuo.toml"),
        r#"
[source]
text = "file"
"#,
    )?;

    cmd.stdin(Stdio::piped());

    // workaround for it not being super easy to pipe data to stdin

    let mut output = cmd.spawn()?;
    let stdin = output.stdin.as_mut().unwrap();
    stdin.write_all(
        r#"
[source]
text = "stdin"
"#
        .as_bytes(),
    )?;
    let output = output.wait_with_output()?;
    let assert = Assert::new(output).append_context("command", format!("{:?}", cmd.deref()));

    assert.success().stdout(predicate::eq("stdin"));

    Ok(())
}

// == helpers ==

/// Returns a type that looks like a Command, but works entirely within a temporary directory.
/// When this is dropped, it will drop both the `Command` and the `TempDir` that it resides in.
fn cmd() -> Result<CommandWithTempWorksapce, Box<dyn std::error::Error>> {
    let workspace = TempDir::new()?;
    let mut command = std::process::Command::cargo_bin("assuo")?;

    command.current_dir(&workspace);
    Ok(CommandWithTempWorksapce { command, workspace })
}

// all of this is just boilerplate so that TempDir gets dropped after Command is dropped
struct CommandWithTempWorksapce {
    command: std::process::Command,
    /// Important, because once this gets dropped and goes out of scope, the temporary space gets deleted.
    workspace: TempDir,
}

impl Deref for CommandWithTempWorksapce {
    type Target = std::process::Command;

    fn deref(&self) -> &Self::Target {
        &self.command
    }
}

impl DerefMut for CommandWithTempWorksapce {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.command
    }
}
