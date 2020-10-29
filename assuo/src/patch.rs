//! This module contains all algorithm related things for applying patches.

use crate::models::{AssuoFile, AssuoPatch, Direction};

/// Given an AssuoFile, will perform all patches on the given assuo file and return the patched file.
pub fn do_patch(file: AssuoFile) -> Vec<u8> {
    // in the future, it would be nice to be able to apply patches as they come along so that everything is
    // non-blocking and fast, but for now, it's much simpler to "resolve everything -> apply patches"

    // resolve the base
    let mut file = file.resolve();

    // resolve every patch
    let patches = file
        .patch
        .unwrap_or_default()
        .into_iter()
        .map(|p| p.resolve())
        .collect::<Vec<_>>();

    // so right now i'm just going for simplicity rather than speed, so i just need a method that works for these patches
    // one ideal thing to do is to maintain another Vec with a Vec of indexes that is in the original file
    // really bad in terms of performance, *but* it is simple for finding the index something should be at

    let mut indexes = Vec::with_capacity(file.source.len());
    for i in 0..file.source.len() {
        indexes.push(vec![i]);
    }

    fn get_index(indexes: &Vec<Vec<usize>>, i: usize) -> usize {
        for (idx, index) in indexes.iter().enumerate() {
            if index.contains(&i) {
                return idx;
            }
        }

        panic!("assuo patch out of bounds?");
    }

    // now, we apply each patch sequentially, maintaining the indexes vec as we go
    for patch in patches {
        match patch {
            AssuoPatch::Insert { way, spot, source } => {
                let insertion_point = get_index(&indexes, spot);

                let insertion_point = match way {
                    Direction::Pre => insertion_point,
                    Direction::Post => insertion_point + 1,
                };

                indexes.splice(
                    insertion_point..insertion_point,
                    (0..source.len()).map(|_| vec![std::usize::MAX]),
                );

                file.source.splice(insertion_point..insertion_point, source);
            }
            AssuoPatch::Remove { way, spot, count } => {
                let insertion_point = get_index(&indexes, spot);

                let insertion_point = match way {
                    Direction::Post => insertion_point + 1,
                    Direction::Pre => insertion_point - count,
                };

                let fold = indexes[insertion_point..(insertion_point + count)]
                    .iter()
                    .fold(Vec::new(), |mut acc, elem| {
                        for element in elem {
                            if !acc.contains(element) {
                                acc.push(*element);
                            }
                        }
                        acc
                    });

                indexes.splice(insertion_point..(insertion_point + count), vec![fold]);

                file.source
                    .splice(insertion_point..(insertion_point + count), vec![]);
            }
        }
    }

    file.source
}
