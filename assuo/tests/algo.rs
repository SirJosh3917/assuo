//! Tests for the patching algorithm of `assuo`

use assuo::{
    models::{AssuoFile, AssuoPatch, AssuoSource, Direction},
    patch::do_patch,
};

use rand::seq::SliceRandom;
use rand::thread_rng;

/// This simple test ensures that an insert at a specific spot will insert the data there.
/// It doesn't matter whether we have a pre insert or a post insert, since there is only one,
/// it will insert itself at the `spot` exactly.
#[tokio::test]
async fn single_insert_inserts_at_spot() -> Result<(), Box<dyn std::error::Error>> {
    let file = AssuoFile {
        source: AssuoSource::Text(String::from("Hello!")),
        patch: Some(vec![AssuoPatch::Insert {
            way: Direction::Post,
            spot: "Hello".len(),
            source: AssuoSource::Text(String::from(", World")),
        }]),
    };

    let patched = do_patch(file).await?;

    assert_eq!(&patched, &"Hello, World!".as_bytes());

    let file = AssuoFile {
        source: AssuoSource::Text(String::from("Hello!")),
        patch: Some(vec![AssuoPatch::Insert {
            way: Direction::Pre,
            spot: "Hello".len(),
            source: AssuoSource::Text(String::from(", World")),
        }]),
    };

    let patched = do_patch(file).await?;

    assert_eq!(&patched, &"Hello, World!".as_bytes());
    Ok(())
}

/// Insertions are executed sequentially. First, the "World" should be inserted after the "o", and
/// then the ", " should be inserted next.
#[tokio::test]
async fn two_post_inserts_insert_in_order() -> Result<(), Box<dyn std::error::Error>> {
    let file = AssuoFile {
        source: AssuoSource::Text(String::from("Hello!")),
        patch: Some(vec![
            AssuoPatch::Insert {
                way: Direction::Post,
                spot: "Hello".len(),
                source: AssuoSource::Text(String::from("World")),
            },
            AssuoPatch::Insert {
                way: Direction::Post,
                spot: "Hello".len(),
                source: AssuoSource::Text(String::from(", ")),
            },
        ]),
    };

    let patched = do_patch(file).await?;

    assert_eq!(&patched, &"Hello, World!".as_bytes());
    Ok(())
}

/// Insertions are executed sequentially. Since we're doing pre inserts, we're going to end up inserting
/// before the !. As such, first we pre-insert ", ", and then pre-insert "World".
#[tokio::test]
async fn two_pre_inserts_insert_in_order() -> Result<(), Box<dyn std::error::Error>> {
    let file = AssuoFile {
        source: AssuoSource::Text(String::from("Hello!")),
        patch: Some(vec![
            AssuoPatch::Insert {
                way: Direction::Pre,
                spot: "Hello".len(),
                source: AssuoSource::Text(String::from(", ")),
            },
            AssuoPatch::Insert {
                way: Direction::Pre,
                spot: "Hello".len(),
                source: AssuoSource::Text(String::from("World")),
            },
        ]),
    };

    let patched = do_patch(file).await?;

    assert_eq!(&patched, &"Hello, World!".as_bytes());
    Ok(())
}

/// This test makes sure that all inserts are relative to the source. In this example, we insert
/// the odd characters of "Hello, World!" into the even characters of "Hello, World!".
///
/// The order we insert in is completely randomized as to try catch errors.
#[tokio::test]
async fn inserts_are_relative_to_original_source_randomized(
) -> Result<(), Box<dyn std::error::Error>> {
    for _ in 0..1000 {
        let mut patches = vec![
            AssuoPatch::Insert {
                // #1
                way: Direction::Post,
                spot: "H".len(),
                source: AssuoSource::Text(String::from("e")),
            },
            AssuoPatch::Insert {
                // #2
                way: Direction::Post,
                spot: "Hl".len(),
                source: AssuoSource::Text(String::from("l")),
            },
            AssuoPatch::Insert {
                // #3
                way: Direction::Post,
                spot: "Hlo".len(),
                source: AssuoSource::Text(String::from(",")),
            },
            AssuoPatch::Insert {
                // #4
                way: Direction::Post,
                spot: "Hlo ".len(),
                source: AssuoSource::Text(String::from("W")),
            },
            AssuoPatch::Insert {
                // #5
                way: Direction::Post,
                spot: "Hlo o".len(),
                source: AssuoSource::Text(String::from("r")),
            },
            AssuoPatch::Insert {
                // #6
                way: Direction::Post,
                spot: "Hlo ol".len(),
                source: AssuoSource::Text(String::from("d")),
            },
        ];

        patches.shuffle(&mut thread_rng());

        let file = AssuoFile {
            source: AssuoSource::Text(String::from("Hlo ol!")),
            patch: Some(patches),
        };

        let source = format!("{:?}", file);
        let patched = do_patch(file).await?;

        assert_eq!(&patched, &"Hello, World!".as_bytes(), "{}", source);
    }

    Ok(())
}

/// It shouldn't matter which order we have these pre and post inserts in, because the spot they insert at
/// is relative to the original document, and them being pre and post inserts should make the order clear.
#[tokio::test]
async fn mixed_pre_and_post_inserts_are_in_order() -> Result<(), Box<dyn std::error::Error>> {
    let file = AssuoFile {
        source: AssuoSource::Text(String::from("Hello!")),
        patch: Some(vec![
            AssuoPatch::Insert {
                way: Direction::Pre,
                spot: "Hello".len(),
                source: AssuoSource::Text(String::from("World")),
            },
            AssuoPatch::Insert {
                way: Direction::Post,
                spot: "Hello".len(),
                source: AssuoSource::Text(String::from(", ")),
            },
        ]),
    };

    let patched = do_patch(file).await?;

    assert_eq!(&patched, &"Hello, World!".as_bytes());

    let file = AssuoFile {
        source: AssuoSource::Text(String::from("Hello!")),
        patch: Some(vec![
            AssuoPatch::Insert {
                way: Direction::Post,
                spot: "Hello".len(),
                source: AssuoSource::Text(String::from(", ")),
            },
            AssuoPatch::Insert {
                way: Direction::Pre,
                spot: "Hello".len(),
                source: AssuoSource::Text(String::from("World")),
            },
        ]),
    };

    let patched = do_patch(file).await?;

    assert_eq!(&patched, &"Hello, World!".as_bytes());
    Ok(())
}

// == CODE SAMPLE TESTS ==
// if there is an assuo config file on the documentation, it should be copied here to ensure it works.
// old tests shouldn't get removed (unless there is a `major` version upgrade) to ensure no regressions.

async fn helper<B: AsRef<[u8]>>(
    expected: B,
    source: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let source = assuo::models::try_parse(source)?;
    let result = do_patch(source).await?;
    assert_eq!(expected.as_ref(), result.as_slice());
    Ok(())
}

#[tokio::test]
async fn readme_1() -> Result<(), Box<dyn std::error::Error>> {
    helper(
        "Hello, World!",
        r#"
# The source specifies where we get a copy of the source bytes from.
# In our case, we will specify some `text`.
[source]
text = "Hello!"

# Now, we can apply a series of patches. In TOML, the double brackets
# ([[]]) mean an array, so we could add more of these sequentially in
# the file later.
[[patch]]

# We specify what we want to do. Here, we'll say that we want to insert
# some text. We'll make the source go from "Hello!" to "Hello, World!"
do = "insert"

# Way means the direction we insert from. In our small example, this
# doesn't matter, but we'll cover it more later.
way = "post"

# The spot is where we want to insert at. We can find out where we want
# to insert at based on the index in the source bytes.
#
# | H | e | l | l | o | ! |
# ^   ^   ^   ^   ^   ^   ^
# 0   1   2   3   4   5   6
spot = 5

# Now, we have to supply the source for where we get the bytes from.
# You may notice that this looks very similar to the [source] we have at
# the top, and you'd be absolutely right! Any valid [source] is a valid source
# here too.
source = { text = ", World" }
"#,
    )
    .await
}

#[tokio::test]
async fn readme_2() -> Result<(), Box<dyn std::error::Error>> {
    helper(
        vec![1, 2, 3, 4],
        r#"
[source]
bytes = [1, 2, 3, 4]
"#,
    )
    .await
}

#[tokio::test]
async fn readme_3() -> Result<(), Box<dyn std::error::Error>> {
    helper(
        "Hello!",
        r#"
[source]
text = "Hello!"
"#,
    )
    .await
}

// TODO: test loading files
// TODO: test loading URL
// TODO: test loading assuo files
// TODO: test loading assuo URL

#[tokio::test]
async fn readme_8() -> Result<(), Box<dyn std::error::Error>> {
    helper(
        ">ba<",
        r#"
[source]
text = "><"

[[patch]]
do = "insert"
way = "post"
spot = 1
source = { text = "a" }

[[patch]]
do = "insert"
way = "post"
spot = 1
source = { text = "b" }
"#,
    )
    .await
}

#[tokio::test]
async fn readme_9() -> Result<(), Box<dyn std::error::Error>> {
    helper(
        ">bac<",
        r#"
[source]
text = "><"

[[patch]]
do = "insert"
way = "post"
spot = 1
source = { text = "a" }

[[patch]]
do = "insert"
way = "post"
spot = 1
source = { text = "b" }

[[patch]]
do = "insert"
way = "pre"
spot = 1
source = { text = "c" }
"#,
    )
    .await
}

// == PREVENTING REGRESSION TESTS ==
// if there is an issue posted, a test should be placed after this point to ensure that there will be no regression in the future
