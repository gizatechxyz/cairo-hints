use std::vec::IntoIter;

use cairo_lang_runner::casm_run::format_next_item;
use cairo_lang_utils::byte_array::BYTE_ARRAY_MAGIC;
use cairo_vm::Felt252;
use itertools::Itertools;

use crate::{TestCompilation, TestCompiler};

#[macro_export]
macro_rules! felt_str {
    ($val:expr) => {
        Felt252::from_str($val).expect("Couldn't parse string")
    };
    ($val:expr, hex) => {
        Felt252::from_hex_unchecked($val)
    };
}

/// Formats the given felts as a panic string.
fn format_for_panic(mut felts: IntoIter<Felt252>) -> String {
    let mut items = Vec::new();
    while let Some(item) = format_next_item(&mut felts) {
        items.push(item.quote_if_string());
    }
    let panic_values_string = if let [item] = &items[..] {
        item.clone()
    } else {
        format!("({})", items.join(", "))
    };
    format!("Panicked with {panic_values_string}.")
}

#[test]
fn test_compiled_serialization() {
    use std::path::PathBuf;
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_data");

    let compiler = TestCompiler::try_new(&path, true).unwrap();
    let compiled = compiler.build().unwrap();
    let serialized = serde_json::to_string_pretty(&compiled).unwrap();
    let deserialized: TestCompilation = serde_json::from_str(&serialized).unwrap();

    assert_eq!(compiled.sierra_program, deserialized.sierra_program);
    assert_eq!(
        compiled.metadata.function_set_costs,
        deserialized.metadata.function_set_costs
    );
    assert_eq!(
        compiled.metadata.named_tests,
        deserialized.metadata.named_tests
    );
    assert_eq!(
        compiled.metadata.contracts_info.values().collect_vec(),
        deserialized.metadata.contracts_info.values().collect_vec()
    );
}

#[test]
fn test_format_for_panic() {
    // Valid short string.
    let felts = vec![felt_str!("68656c6c6f", hex)];
    assert_eq!(
        format_for_panic(felts.into_iter()),
        "Panicked with 0x68656c6c6f ('hello')."
    );

    // felt252
    let felts = vec![Felt252::from(1)];
    assert_eq!(format_for_panic(felts.into_iter()), "Panicked with 0x1.");

    // Valid string with < 31 characters (no full words).
    let felts = vec![
        felt_str!(BYTE_ARRAY_MAGIC, hex),
        Felt252::from(0),                                     // No full words.
        felt_str!("73686f72742c2062757420737472696e67", hex), // 'short, but string'
        Felt252::from(17),                                    // pending word length
    ];
    assert_eq!(
        format_for_panic(felts.into_iter()),
        "Panicked with \"short, but string\"."
    );

    // Valid string with > 31 characters (with a full word).
    let felts = vec![
        felt_str!(BYTE_ARRAY_MAGIC, hex),
        // A single full word.
        Felt252::from(1),
        // full word: 'This is a long string with more'
        felt_str!(
            "546869732069732061206c6f6e6720737472696e672077697468206d6f7265",
            hex
        ),
        // pending word: ' than 31 characters.'
        felt_str!("207468616e20333120636861726163746572732e", hex),
        // pending word length
        Felt252::from(20),
    ];
    assert_eq!(
        format_for_panic(felts.into_iter()),
        "Panicked with \"This is a long string with more than 31 characters.\"."
    );

    // Only magic.
    let felts = vec![felt_str!(BYTE_ARRAY_MAGIC, hex)];
    assert_eq!(
        format_for_panic(felts.into_iter()),
        "Panicked with 0x46a6158a16a947e5916b2a2ca68501a45e93d7110e81aa2d6438b1c57c879a3."
    );

    // num_full_words > usize.
    let felts = vec![
        felt_str!(BYTE_ARRAY_MAGIC, hex),
        felt_str!("100000000", hex),
    ];
    assert_eq!(
        format_for_panic(felts.into_iter()),
        "Panicked with (0x46a6158a16a947e5916b2a2ca68501a45e93d7110e81aa2d6438b1c57c879a3, \
         0x100000000)."
    );

    // Not enough data after num_full_words.
    let felts = vec![felt_str!(BYTE_ARRAY_MAGIC, hex), Felt252::from(0)];
    assert_eq!(
        format_for_panic(felts.into_iter()),
        "Panicked with (0x46a6158a16a947e5916b2a2ca68501a45e93d7110e81aa2d6438b1c57c879a3, 0x0 \
         (''))."
    );

    // Not enough full words.
    let felts = vec![
        felt_str!(BYTE_ARRAY_MAGIC, hex),
        Felt252::from(1),
        Felt252::from(0),
        Felt252::from(0),
    ];
    assert_eq!(
        format_for_panic(felts.into_iter()),
        "Panicked with (0x46a6158a16a947e5916b2a2ca68501a45e93d7110e81aa2d6438b1c57c879a3, 0x1, \
         0x0 (''), 0x0 (''))."
    );

    // Too much data in full word.
    let felts = vec![
        felt_str!(BYTE_ARRAY_MAGIC, hex),
        Felt252::from(1),
        felt_str!(
            "161616161616161616161616161616161616161616161616161616161616161",
            hex
        ),
        Felt252::from(0),
        Felt252::from(0),
    ];
    assert_eq!(
        format_for_panic(felts.into_iter()),
        "Panicked with (0x46a6158a16a947e5916b2a2ca68501a45e93d7110e81aa2d6438b1c57c879a3, 0x1, \
         0x161616161616161616161616161616161616161616161616161616161616161, 0x0 (''), 0x0 (''))."
    );

    // num_pending_bytes > usize.
    let felts = vec![
        felt_str!(BYTE_ARRAY_MAGIC, hex),
        Felt252::from(0),
        Felt252::from(0),
        felt_str!("100000000", hex),
    ];
    assert_eq!(
        format_for_panic(felts.into_iter()),
        "Panicked with (0x46a6158a16a947e5916b2a2ca68501a45e93d7110e81aa2d6438b1c57c879a3, 0x0 \
         (''), 0x0 (''), 0x100000000)."
    );

    // "Not enough" data in pending_word (nulls in the beginning).
    let felts = vec![
        felt_str!(BYTE_ARRAY_MAGIC, hex),
        // No full words.
        Felt252::from(0),
        // 'a'
        felt_str!("61", hex),
        // pending word length. Bigger than the actual data in the pending word.
        Felt252::from(2),
    ];
    assert_eq!(
        format_for_panic(felts.into_iter()),
        "Panicked with \"\\0a\"."
    );

    // Too much data in pending_word.
    let felts = vec![
        felt_str!(BYTE_ARRAY_MAGIC, hex),
        // No full words.
        Felt252::from(0),
        // 'aa'
        felt_str!("6161", hex),
        // pending word length. Smaller than the actual data in the pending word.
        Felt252::from(1),
    ];
    assert_eq!(
        format_for_panic(felts.into_iter()),
        "Panicked with (0x46a6158a16a947e5916b2a2ca68501a45e93d7110e81aa2d6438b1c57c879a3, 0x0 \
         (''), 0x6161 ('aa'), 0x1)."
    );

    // Valid string with Null.
    let felts = vec![
        felt_str!(BYTE_ARRAY_MAGIC, hex),
        // No full word.
        Felt252::from(0),
        // pending word: 'Hello\0world'
        felt_str!("48656c6c6f00776f726c64", hex),
        // pending word length
        Felt252::from(11),
    ];
    assert_eq!(
        format_for_panic(felts.into_iter()),
        "Panicked with \"Hello\\0world\"."
    );

    // Valid string with a non printable character.
    let felts = vec![
        felt_str!(BYTE_ARRAY_MAGIC, hex),
        // No full word.
        Felt252::from(0),
        // pending word: 'Hello\x11world'
        felt_str!("48656c6c6f11776f726c64", hex),
        // pending word length
        Felt252::from(11),
    ];
    assert_eq!(
        format_for_panic(felts.into_iter()),
        "Panicked with \"Hello\\x11world\"."
    );

    // Valid string with a newline.
    let felts = vec![
        felt_str!(BYTE_ARRAY_MAGIC, hex),
        // No full word.
        Felt252::from(0),
        // pending word: 'Hello\nworld'
        felt_str!("48656c6c6f0a776f726c64", hex),
        // pending word length
        Felt252::from(11),
    ];
    assert_eq!(
        format_for_panic(felts.into_iter()),
        "Panicked with \"Hello\nworld\"."
    );

    // Multiple values: (felt, string, short_string, felt)
    let felts = vec![
        // felt: 0x9999
        Felt252::from(0x9999),
        // String: "hello"
        felt_str!(BYTE_ARRAY_MAGIC, hex),
        Felt252::from(0),
        felt_str!("68656c6c6f", hex),
        Felt252::from(5),
        // Short string: 'world'
        felt_str!("776f726c64", hex),
        // felt: 0x8888
        Felt252::from(0x8888),
    ];
    assert_eq!(
        format_for_panic(felts.into_iter()),
        "Panicked with (0x9999, \"hello\", 0x776f726c64 ('world'), 0x8888)."
    );
}
