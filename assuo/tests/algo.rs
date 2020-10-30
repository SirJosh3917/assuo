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
