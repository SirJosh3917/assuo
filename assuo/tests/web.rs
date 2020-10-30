//! Houses all tests that need a web server for whatever reason

use assuo::models::try_parse;
use assuo::models::Resolvable;
use httptest::{matchers::request, responders::status_code, Expectation, Server};

#[tokio::test]
async fn when_source_is_url_it_resolves_it_by_downloading_it(
) -> Result<(), Box<dyn std::error::Error>> {
    let server = Server::run();

    server.expect(
        Expectation::matching(request::method_path("GET", "/"))
            .respond_with(status_code(200).body("Hello, World!")),
    );

    let url = server.url("/");

    let assuo_config = try_parse(&format!(
        r#"
[source]
url = "{}"
"#,
        url
    ))
    .unwrap();

    let resolved = assuo_config.resolve().await?;
    assert_eq!(resolved.source.as_slice(), "Hello, World!".as_bytes());

    Ok(())
}

#[tokio::test]
async fn when_source_is_assuo_url_it_resolves_it_by_downloading_it(
) -> Result<(), Box<dyn std::error::Error>> {
    let server = Server::run();

    server.expect(
        Expectation::matching(request::method_path("GET", "/")).respond_with(
            status_code(200).body(
                r#"
[source]
text = "Hello, World!"
"#,
            ),
        ),
    );

    let url = server.url("/");

    let assuo_config = try_parse(&format!(
        r#"
[source]
assuo-url = "{}"
"#,
        url
    ))
    .unwrap();

    let resolved = assuo_config.resolve().await?;
    assert_eq!(resolved.source.as_slice(), "Hello, World!".as_bytes());

    Ok(())
}
