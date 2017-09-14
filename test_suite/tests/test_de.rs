// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[macro_use]
extern crate serde_derive;

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::net;
use std::path::{Path, PathBuf};
use std::time::{Duration, UNIX_EPOCH};
use std::default::Default;
use std::ffi::{CString, OsString};
use std::rc::Rc;
use std::sync::Arc;

#[cfg(feature = "unstable")]
use std::ffi::CStr;

extern crate serde;
use serde::Deserialize;

extern crate fnv;
use self::fnv::FnvHasher;

extern crate serde_test;
use self::serde_test::{Token, assert_de_tokens, assert_de_tokens_error, assert_de_tokens_readable};

#[macro_use]
mod macros;

//////////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, PartialEq, Debug, Deserialize)]
struct UnitStruct;

#[derive(PartialEq, Debug, Deserialize)]
struct NewtypeStruct(i32);

#[derive(PartialEq, Debug, Deserialize)]
struct TupleStruct(i32, i32, i32);

#[derive(PartialEq, Debug, Deserialize)]
struct Struct {
    a: i32,
    b: i32,
    #[serde(skip_deserializing)]
    c: i32,
}

#[derive(PartialEq, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StructDenyUnknown {
    a: i32,
    #[serde(skip_deserializing)]
    b: i32,
}

#[derive(PartialEq, Debug, Deserialize)]
#[serde(default)]
struct StructDefault<T> {
    a: i32,
    b: T,
}

impl Default for StructDefault<String> {
    fn default() -> Self {
        StructDefault {
            a: 100,
            b: "default".to_string(),
        }
    }
}

#[derive(PartialEq, Debug, Deserialize)]
struct StructSkipAll {
    #[serde(skip_deserializing)]
    a: i32,
}

#[derive(PartialEq, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StructSkipAllDenyUnknown {
    #[serde(skip_deserializing)]
    a: i32,
}

#[derive(PartialEq, Debug, Deserialize)]
enum Enum {
    #[allow(dead_code)]
    #[serde(skip_deserializing)]
    Skipped,
    Unit,
    Simple(i32),
    Seq(i32, i32, i32),
    Map { a: i32, b: i32, c: i32 },
}

#[derive(PartialEq, Debug, Deserialize)]
enum EnumSkipAll {
    #[allow(dead_code)]
    #[serde(skip_deserializing)]
    Skipped,
}

//////////////////////////////////////////////////////////////////////////

macro_rules! declare_test {
    ($name:ident $readable: ident { $($value:expr => $tokens:expr,)+ }) => {
        #[test]
        fn $name() {
            $(
                // Test ser/de roundtripping
                assert_de_tokens_readable(&$value, $tokens, $readable);

                // Test that the tokens are ignorable
                assert_de_tokens_ignore($tokens, true);
            )+
        }
    }
}

macro_rules! declare_tests {
    ($($name:ident { $($value:expr => $tokens:expr,)+ })+) => {
        $(
            declare_test!($name true { $($value => $tokens,)+ });
        )+
    }
}

macro_rules! declare_error_tests {
    ($($name:ident<$target:ty> { $tokens:expr, $expected:expr, })+) => {
        $(
            #[test]
            fn $name() {
                assert_de_tokens_error::<$target>($tokens, $expected);
            }
        )+
    }
}

macro_rules! declare_non_human_readable_tests {
    ($($name:ident { $($value:expr => $tokens:expr,)+ })+) => {
        $(
            declare_test!($name false { $($value => $tokens,)+ });
        )+
    }
}


fn assert_de_tokens_ignore(ignorable_tokens: &[Token], readable: bool) {
    #[derive(PartialEq, Debug, Deserialize)]
    struct IgnoreBase {
        a: i32,
    }

    // Embed the tokens to be ignored in the normal token
    // stream for an IgnoreBase type
    let concated_tokens: Vec<Token> = vec![
        Token::Map { len: Some(2) },
        Token::Str("a"),
        Token::I32(1),

        Token::Str("ignored"),
    ]
            .into_iter()
            .chain(ignorable_tokens.to_vec().into_iter())
            .chain(vec![Token::MapEnd].into_iter())
            .collect();

    let mut de = serde_test::Deserializer::readable(&concated_tokens, readable);
    let base = IgnoreBase::deserialize(&mut de).unwrap();
    assert_eq!(base, IgnoreBase { a: 1 });
}

//////////////////////////////////////////////////////////////////////////

declare_tests! {
    test_bool {
        true => &[Token::Bool(true)],
        false => &[Token::Bool(false)],
    }
    test_isize {
        0isize => &[Token::I8(0)],
        0isize => &[Token::I16(0)],
        0isize => &[Token::I32(0)],
        0isize => &[Token::I64(0)],
        0isize => &[Token::U8(0)],
        0isize => &[Token::U16(0)],
        0isize => &[Token::U32(0)],
        0isize => &[Token::U64(0)],
    }
    test_ints {
        0i8 => &[Token::I8(0)],
        0i16 => &[Token::I16(0)],
        0i32 => &[Token::I32(0)],
        0i64 => &[Token::I64(0)],
    }
    test_uints {
        0u8 => &[Token::U8(0)],
        0u16 => &[Token::U16(0)],
        0u32 => &[Token::U32(0)],
        0u64 => &[Token::U64(0)],
    }
    test_floats {
        0f32 => &[Token::F32(0.)],
        0f64 => &[Token::F64(0.)],
    }
    test_char {
        'a' => &[Token::Char('a')],
        'a' => &[Token::Str("a")],
        'a' => &[Token::String("a")],
    }
    test_string {
        "abc".to_owned() => &[Token::Str("abc")],
        "abc".to_owned() => &[Token::String("abc")],
        "a".to_owned() => &[Token::Char('a')],
    }
    test_option {
        None::<i32> => &[Token::Unit],
        None::<i32> => &[Token::None],
        Some(1) => &[
            Token::Some,
            Token::I32(1),
        ],
    }
    test_result {
        Ok::<i32, i32>(0) => &[
            Token::Enum { name: "Result" },
            Token::Str("Ok"),
            Token::I32(0),
        ],
        Err::<i32, i32>(1) => &[
            Token::Enum { name: "Result" },
            Token::Str("Err"),
            Token::I32(1),
        ],
    }
    test_unit {
        () => &[Token::Unit],
    }
    test_unit_struct {
        UnitStruct => &[Token::Unit],
        UnitStruct => &[
            Token::UnitStruct { name: "UnitStruct" },
        ],
    }
    test_newtype_struct {
        NewtypeStruct(1) => &[
            Token::NewtypeStruct { name: "NewtypeStruct" },
            Token::I32(1),
        ],
    }
    test_tuple_struct {
        TupleStruct(1, 2, 3) => &[
            Token::Seq { len: Some(3) },
                Token::I32(1),
                Token::I32(2),
                Token::I32(3),
            Token::SeqEnd,
        ],
        TupleStruct(1, 2, 3) => &[
            Token::Seq { len: None },
                Token::I32(1),
                Token::I32(2),
                Token::I32(3),
            Token::SeqEnd,
        ],
        TupleStruct(1, 2, 3) => &[
            Token::TupleStruct { name: "TupleStruct", len: 3 },
                Token::I32(1),
                Token::I32(2),
                Token::I32(3),
            Token::TupleStructEnd,
        ],
        TupleStruct(1, 2, 3) => &[
            Token::TupleStruct { name: "TupleStruct", len: 3 },
                Token::I32(1),
                Token::I32(2),
                Token::I32(3),
            Token::TupleStructEnd,
        ],
    }
    test_btreeset {
        BTreeSet::<isize>::new() => &[
            Token::Seq { len: Some(0) },
            Token::SeqEnd,
        ],
        btreeset![btreeset![], btreeset![1], btreeset![2, 3]] => &[
            Token::Seq { len: Some(3) },
                Token::Seq { len: Some(0) },
                Token::SeqEnd,

                Token::Seq { len: Some(1) },
                    Token::I32(1),
                Token::SeqEnd,

                Token::Seq { len: Some(2) },
                    Token::I32(2),
                    Token::I32(3),
                Token::SeqEnd,
            Token::SeqEnd,
        ],
        BTreeSet::<isize>::new() => &[
            Token::TupleStruct { name: "Anything", len: 0 },
            Token::TupleStructEnd,
        ],
    }
    test_hashset {
        HashSet::<isize>::new() => &[
            Token::Seq { len: Some(0) },
            Token::SeqEnd,
        ],
        hashset![1, 2, 3] => &[
            Token::Seq { len: Some(3) },
                Token::I32(1),
                Token::I32(2),
                Token::I32(3),
            Token::SeqEnd,
        ],
        HashSet::<isize>::new() => &[
            Token::TupleStruct { name: "Anything", len: 0 },
            Token::TupleStructEnd,
        ],
        hashset![FnvHasher @ 1, 2, 3] => &[
            Token::Seq { len: Some(3) },
                Token::I32(1),
                Token::I32(2),
                Token::I32(3),
            Token::SeqEnd,
        ],
    }
    test_vec {
        Vec::<isize>::new() => &[
            Token::Seq { len: Some(0) },
            Token::SeqEnd,
        ],
        vec![vec![], vec![1], vec![2, 3]] => &[
            Token::Seq { len: Some(3) },
                Token::Seq { len: Some(0) },
                Token::SeqEnd,

                Token::Seq { len: Some(1) },
                    Token::I32(1),
                Token::SeqEnd,

                Token::Seq { len: Some(2) },
                    Token::I32(2),
                    Token::I32(3),
                Token::SeqEnd,
            Token::SeqEnd,
        ],
        Vec::<isize>::new() => &[
            Token::TupleStruct { name: "Anything", len: 0 },
            Token::TupleStructEnd,
        ],
    }
    test_array {
        [0; 0] => &[
            Token::Seq { len: Some(0) },
            Token::SeqEnd,
        ],
        [0; 0] => &[
            Token::Tuple { len: 0 },
            Token::TupleEnd,
        ],
        ([0; 0], [1], [2, 3]) => &[
            Token::Seq { len: Some(3) },
                Token::Seq { len: Some(0) },
                Token::SeqEnd,

                Token::Seq { len: Some(1) },
                    Token::I32(1),
                Token::SeqEnd,

                Token::Seq { len: Some(2) },
                    Token::I32(2),
                    Token::I32(3),
                Token::SeqEnd,
            Token::SeqEnd,
        ],
        ([0; 0], [1], [2, 3]) => &[
            Token::Tuple { len: 3 },
                Token::Tuple { len: 0 },
                Token::TupleEnd,

                Token::Tuple { len: 1 },
                    Token::I32(1),
                Token::TupleEnd,

                Token::Tuple { len: 2 },
                    Token::I32(2),
                    Token::I32(3),
                Token::TupleEnd,
            Token::TupleEnd,
        ],
        [0; 0] => &[
            Token::TupleStruct { name: "Anything", len: 0 },
            Token::TupleStructEnd,
        ],
    }
    test_tuple {
        (1,) => &[
            Token::Seq { len: Some(1) },
                Token::I32(1),
            Token::SeqEnd,
        ],
        (1, 2, 3) => &[
            Token::Seq { len: Some(3) },
                Token::I32(1),
                Token::I32(2),
                Token::I32(3),
            Token::SeqEnd,
        ],
        (1,) => &[
            Token::Tuple { len: 1 },
                Token::I32(1),
            Token::TupleEnd,
        ],
        (1, 2, 3) => &[
            Token::Tuple { len: 3 },
                Token::I32(1),
                Token::I32(2),
                Token::I32(3),
            Token::TupleEnd,
        ],
    }
    test_btreemap {
        BTreeMap::<isize, isize>::new() => &[
            Token::Map { len: Some(0) },
            Token::MapEnd,
        ],
        btreemap![1 => 2] => &[
            Token::Map { len: Some(1) },
                Token::I32(1),
                Token::I32(2),
            Token::MapEnd,
        ],
        btreemap![1 => 2, 3 => 4] => &[
            Token::Map { len: Some(2) },
                Token::I32(1),
                Token::I32(2),

                Token::I32(3),
                Token::I32(4),
            Token::MapEnd,
        ],
        btreemap![1 => btreemap![], 2 => btreemap![3 => 4, 5 => 6]] => &[
            Token::Map { len: Some(2) },
                Token::I32(1),
                Token::Map { len: Some(0) },
                Token::MapEnd,

                Token::I32(2),
                Token::Map { len: Some(2) },
                    Token::I32(3),
                    Token::I32(4),

                    Token::I32(5),
                    Token::I32(6),
                Token::MapEnd,
            Token::MapEnd,
        ],
        BTreeMap::<isize, isize>::new() => &[
            Token::Struct { name: "Anything", len: 0 },
            Token::StructEnd,
        ],
    }
    test_hashmap {
        HashMap::<isize, isize>::new() => &[
            Token::Map { len: Some(0) },
            Token::MapEnd,
        ],
        hashmap![1 => 2] => &[
            Token::Map { len: Some(1) },
                Token::I32(1),
                Token::I32(2),
            Token::MapEnd,
        ],
        hashmap![1 => 2, 3 => 4] => &[
            Token::Map { len: Some(2) },
                Token::I32(1),
                Token::I32(2),

                Token::I32(3),
                Token::I32(4),
            Token::MapEnd,
        ],
        hashmap![1 => hashmap![], 2 => hashmap![3 => 4, 5 => 6]] => &[
            Token::Map { len: Some(2) },
                Token::I32(1),
                Token::Map { len: Some(0) },
                Token::MapEnd,

                Token::I32(2),
                Token::Map { len: Some(2) },
                    Token::I32(3),
                    Token::I32(4),

                    Token::I32(5),
                    Token::I32(6),
                Token::MapEnd,
            Token::MapEnd,
        ],
        HashMap::<isize, isize>::new() => &[
            Token::Struct { name: "Anything", len: 0 },
            Token::StructEnd,
        ],
        hashmap![FnvHasher @ 1 => 2, 3 => 4] => &[
            Token::Map { len: Some(2) },
                Token::I32(1),
                Token::I32(2),

                Token::I32(3),
                Token::I32(4),
            Token::MapEnd,
        ],
    }
    test_struct {
        Struct { a: 1, b: 2, c: 0 } => &[
            Token::Map { len: Some(3) },
                Token::Str("a"),
                Token::I32(1),

                Token::Str("b"),
                Token::I32(2),
            Token::MapEnd,
        ],
        Struct { a: 1, b: 2, c: 0 } => &[
            Token::Struct { name: "Struct", len: 3 },
                Token::Str("a"),
                Token::I32(1),

                Token::Str("b"),
                Token::I32(2),
            Token::StructEnd,
        ],
        Struct { a: 1, b: 2, c: 0 } => &[
            Token::Seq { len: Some(3) },
                Token::I32(1),
                Token::I32(2),
            Token::SeqEnd,
        ],
    }
    test_struct_with_skip {
        Struct { a: 1, b: 2, c: 0 } => &[
            Token::Map { len: Some(3) },
                Token::Str("a"),
                Token::I32(1),

                Token::Str("b"),
                Token::I32(2),

                Token::Str("c"),
                Token::I32(3),

                Token::Str("d"),
                Token::I32(4),
            Token::MapEnd,
        ],
        Struct { a: 1, b: 2, c: 0 } => &[
            Token::Struct { name: "Struct", len: 3 },
                Token::Str("a"),
                Token::I32(1),

                Token::Str("b"),
                Token::I32(2),

                Token::Str("c"),
                Token::I32(3),

                Token::Str("d"),
                Token::I32(4),
            Token::StructEnd,
        ],
    }
    test_struct_skip_all {
        StructSkipAll { a: 0 } => &[
            Token::Struct { name: "StructSkipAll", len: 0 },
            Token::StructEnd,
        ],
        StructSkipAll { a: 0 } => &[
            Token::Struct { name: "StructSkipAll", len: 1 },
                Token::Str("a"),
                Token::I32(1),

                Token::Str("b"),
                Token::I32(2),
            Token::StructEnd,
        ],
    }
    test_struct_skip_all_deny_unknown {
        StructSkipAllDenyUnknown { a: 0 } => &[
            Token::Struct { name: "StructSkipAllDenyUnknown", len: 0 },
            Token::StructEnd,
        ],
    }
    test_struct_default {
        StructDefault { a: 50, b: "overwritten".to_string() } => &[
            Token::Struct { name: "StructDefault", len: 1 },
                Token::Str("a"),
                Token::I32(50),

                Token::Str("b"),
                Token::String("overwritten"),
            Token::StructEnd,
        ],
        StructDefault { a: 100, b: "default".to_string() } => &[
            Token::Struct { name: "StructDefault",  len: 0 },
            Token::StructEnd,
        ],
    }
    test_enum_unit {
        Enum::Unit => &[
            Token::UnitVariant { name: "Enum", variant: "Unit" },
        ],
    }
    test_enum_simple {
        Enum::Simple(1) => &[
            Token::NewtypeVariant { name: "Enum", variant: "Simple" },
            Token::I32(1),
        ],
    }
    test_enum_seq {
        Enum::Seq(1, 2, 3) => &[
            Token::TupleVariant { name: "Enum", variant: "Seq", len: 3 },
                Token::I32(1),
                Token::I32(2),
                Token::I32(3),
            Token::TupleVariantEnd,
        ],
    }
    test_enum_map {
        Enum::Map { a: 1, b: 2, c: 3 } => &[
            Token::StructVariant { name: "Enum", variant: "Map", len: 3 },
                Token::Str("a"),
                Token::I32(1),

                Token::Str("b"),
                Token::I32(2),

                Token::Str("c"),
                Token::I32(3),
            Token::StructVariantEnd,
        ],
    }
    test_enum_unit_usize {
        Enum::Unit => &[
            Token::Enum { name: "Enum" },
            Token::U32(0),
            Token::Unit,
        ],
    }
    test_enum_unit_bytes {
        Enum::Unit => &[
            Token::Enum { name: "Enum" },
            Token::Bytes(b"Unit"),
            Token::Unit,
        ],
    }
    test_box {
        Box::new(0i32) => &[Token::I32(0)],
    }
    test_boxed_slice {
        Box::new([0, 1, 2]) => &[
            Token::Seq { len: Some(3) },
            Token::I32(0),
            Token::I32(1),
            Token::I32(2),
            Token::SeqEnd,
        ],
    }
    test_duration {
        Duration::new(1, 2) => &[
            Token::Struct { name: "Duration", len: 2 },
                Token::Str("secs"),
                Token::U64(1),

                Token::Str("nanos"),
                Token::U32(2),
            Token::StructEnd,
        ],
        Duration::new(1, 2) => &[
            Token::Seq { len: Some(2) },
                Token::I64(1),
                Token::I64(2),
            Token::SeqEnd,
        ],
    }
    test_system_time {
        UNIX_EPOCH + Duration::new(1, 2) => &[
            Token::Struct { name: "SystemTime", len: 2 },
                Token::Str("secs_since_epoch"),
                Token::U64(1),

                Token::Str("nanos_since_epoch"),
                Token::U32(2),
            Token::StructEnd,
        ],
        UNIX_EPOCH + Duration::new(1, 2) => &[
            Token::Seq { len: Some(2) },
                Token::I64(1),
                Token::I64(2),
            Token::SeqEnd,
        ],
    }
    test_range {
        1u32..2u32 => &[
            Token::Struct { name: "Range", len: 2 },
                Token::Str("start"),
                Token::U32(1),

                Token::Str("end"),
                Token::U32(2),
            Token::StructEnd,
        ],
        1u32..2u32 => &[
            Token::Seq { len: Some(2) },
                Token::U64(1),
                Token::U64(2),
            Token::SeqEnd,
        ],
    }
    test_net_ipv4addr {
        "1.2.3.4".parse::<net::Ipv4Addr>().unwrap() => &[Token::Str("1.2.3.4")],
    }
    test_net_ipv6addr {
        "::1".parse::<net::Ipv6Addr>().unwrap() => &[Token::Str("::1")],
    }
    test_net_socketaddr {
        "1.2.3.4:1234".parse::<net::SocketAddr>().unwrap() => &[Token::Str("1.2.3.4:1234")],
        "1.2.3.4:1234".parse::<net::SocketAddrV4>().unwrap() => &[Token::Str("1.2.3.4:1234")],
        "[::1]:1234".parse::<net::SocketAddrV6>().unwrap() => &[Token::Str("[::1]:1234")],
    }
    test_path {
        Path::new("/usr/local/lib") => &[
            Token::BorrowedStr("/usr/local/lib"),
        ],
    }
    test_path_buf {
        PathBuf::from("/usr/local/lib") => &[
            Token::String("/usr/local/lib"),
        ],
    }
    test_cstring {
        CString::new("abc").unwrap() => &[
            Token::Bytes(b"abc"),
        ],
    }
    test_rc {
        Rc::new(true) => &[
            Token::Bool(true),
        ],
    }
    test_arc {
        Arc::new(true) => &[
            Token::Bool(true),
        ],
    }
}

declare_non_human_readable_tests!{
    test_non_human_readable_net_ipv4addr {
        net::Ipv4Addr::from(*b"1234") => &seq![
            Token::Tuple { len: 4 },
            seq b"1234".iter().map(|&b| Token::U8(b)),
            Token::TupleEnd
        ],
    }
    test_non_human_readable_net_ipv6addr {
        net::Ipv6Addr::from(*b"1234567890123456") => &seq![
            Token::Tuple { len: 4 },
            seq b"1234567890123456".iter().map(|&b| Token::U8(b)),
            Token::TupleEnd
        ],

    }
    test_non_human_readable_net_socketaddr {
        net::SocketAddr::from((*b"1234567890123456", 1234)) => &seq![
            Token::NewtypeVariant { name: "SocketAddr", variant: "V6" },

            Token::Tuple { len: 2 },

            Token::Tuple { len: 16 },
            seq b"1234567890123456".iter().map(|&b| Token::U8(b)),
            Token::TupleEnd,

            Token::U16(1234),
            Token::TupleEnd
        ],
        net::SocketAddr::from((*b"1234", 1234)) => &seq![
            Token::NewtypeVariant { name: "SocketAddr", variant: "V4" },

            Token::Tuple { len: 2 },

            Token::Tuple { len: 4 },
            seq b"1234".iter().map(|&b| Token::U8(b)),
            Token::TupleEnd,

            Token::U16(1234),
            Token::TupleEnd
        ],
        net::SocketAddrV4::new(net::Ipv4Addr::from(*b"1234"), 1234) => &seq![
            Token::Tuple { len: 2 },

            Token::Tuple { len: 4 },
            seq b"1234".iter().map(|&b| Token::U8(b)),
            Token::TupleEnd,

            Token::U16(1234),
            Token::TupleEnd
        ],
        net::SocketAddrV6::new(net::Ipv6Addr::from(*b"1234567890123456"), 1234, 0, 0) => &seq![
            Token::Tuple { len: 2 },

            Token::Tuple { len: 16 },
            seq b"1234567890123456".iter().map(|&b| Token::U8(b)),
            Token::TupleEnd,

            Token::U16(1234),
            Token::TupleEnd
        ],
    }
}

#[cfg(feature = "unstable")]
declare_tests! {
    test_rc_dst {
        Rc::<str>::from("s") => &[
            Token::Str("s"),
        ],
        Rc::<[bool]>::from(&[true][..]) => &[
            Token::Seq { len: Some(1) },
            Token::Bool(true),
            Token::SeqEnd,
        ],
    }
    test_arc_dst {
        Arc::<str>::from("s") => &[
            Token::Str("s"),
        ],
        Arc::<[bool]>::from(&[true][..]) => &[
            Token::Seq { len: Some(1) },
            Token::Bool(true),
            Token::SeqEnd,
        ],
    }
}

#[cfg(unix)]
#[test]
fn test_osstring() {
    use std::os::unix::ffi::OsStringExt;

    let value = OsString::from_vec(vec![1, 2, 3]);
    let tokens = [
        Token::Enum { name: "OsString" },
        Token::Str("Unix"),
        Token::Seq { len: Some(2) },
        Token::U8(1),
        Token::U8(2),
        Token::U8(3),
        Token::SeqEnd,
    ];

    assert_de_tokens(&value, &tokens);
    assert_de_tokens_ignore(&tokens, true);
}

#[cfg(windows)]
#[test]
fn test_osstring() {
    use std::os::windows::ffi::OsStringExt;

    let value = OsString::from_wide(&[1, 2, 3]);
    let tokens = [
        Token::Enum { name: "OsString" },
        Token::Str("Windows"),
        Token::Seq { len: Some(2) },
        Token::U16(1),
        Token::U16(2),
        Token::U16(3),
        Token::SeqEnd,
    ];

    assert_de_tokens(&value, &tokens);
    assert_de_tokens_ignore(&tokens, true);
}

#[cfg(feature = "unstable")]
#[test]
fn test_cstr() {
    assert_de_tokens::<Box<CStr>>(
        &CString::new("abc").unwrap().into_boxed_c_str(),
        &[Token::Bytes(b"abc")],
    );
}

#[cfg(feature = "unstable")]
#[test]
fn test_net_ipaddr() {
    assert_de_tokens(
        &"1.2.3.4".parse::<net::IpAddr>().unwrap(),
        &[Token::Str("1.2.3.4")],
    );
}

#[cfg(feature = "unstable")]
#[test]
fn test_cstr_internal_null() {
    assert_de_tokens_error::<Box<CStr>>(
        &[Token::Bytes(b"a\0c")],
        "nul byte found in provided data at position: 1",
    );
}

#[cfg(feature = "unstable")]
#[test]
fn test_cstr_internal_null_end() {
    assert_de_tokens_error::<Box<CStr>>(
        &[Token::Bytes(b"ac\0")],
        "nul byte found in provided data at position: 2",
    );
}

declare_error_tests! {
    test_unknown_field<StructDenyUnknown> {
        &[
            Token::Struct { name: "StructDenyUnknown", len: 2 },
                Token::Str("a"),
                Token::I32(0),

                Token::Str("d"),
        ],
        "unknown field `d`, expected `a`",
    }
    test_skipped_field_is_unknown<StructDenyUnknown> {
        &[
            Token::Struct { name: "StructDenyUnknown", len: 2 },
                Token::Str("b"),
        ],
        "unknown field `b`, expected `a`",
    }
    test_skip_all_deny_unknown<StructSkipAllDenyUnknown> {
        &[
            Token::Struct { name: "StructSkipAllDenyUnknown", len: 1 },
                Token::Str("a"),
        ],
        "unknown field `a`, there are no fields",
    }
    test_unknown_variant<Enum> {
        &[
            Token::UnitVariant { name: "Enum", variant: "Foo" },
        ],
        "unknown variant `Foo`, expected one of `Unit`, `Simple`, `Seq`, `Map`",
    }
    test_enum_skipped_variant<Enum> {
        &[
            Token::UnitVariant { name: "Enum", variant: "Skipped" },
        ],
        "unknown variant `Skipped`, expected one of `Unit`, `Simple`, `Seq`, `Map`",
    }
    test_enum_skip_all<EnumSkipAll> {
        &[
            Token::UnitVariant { name: "EnumSkipAll", variant: "Skipped" },
        ],
        "unknown variant `Skipped`, there are no variants",
    }
    test_duplicate_field_struct<Struct> {
        &[
            Token::Map { len: Some(3) },
                Token::Str("a"),
                Token::I32(1),

                Token::Str("a"),
        ],
        "duplicate field `a`",
    }
    test_duplicate_field_enum<Enum> {
        &[
            Token::StructVariant { name: "Enum", variant: "Map", len: 3 },
                Token::Str("a"),
                Token::I32(1),

                Token::Str("a"),
        ],
        "duplicate field `a`",
    }
    test_enum_out_of_range<Enum> {
        &[
            Token::Enum { name: "Enum" },
            Token::U32(4),
            Token::Unit,
        ],
        "invalid value: integer `4`, expected variant index 0 <= i < 4",
    }
    test_short_tuple<(u8, u8, u8)> {
        &[
            Token::Tuple { len: 1 },
            Token::U8(1),
            Token::TupleEnd,
        ],
        "invalid length 1, expected a tuple of size 3",
    }
    test_short_array<[u8; 3]> {
        &[
            Token::Seq { len: Some(1) },
            Token::U8(1),
            Token::SeqEnd,
        ],
        "invalid length 1, expected an array of length 3",
    }
    test_cstring_internal_null<CString> {
        &[
            Token::Bytes(b"a\0c"),
        ],
        "nul byte found in provided data at position: 1",
    }
    test_cstring_internal_null_end<CString> {
        &[
            Token::Bytes(b"ac\0"),
        ],
        "nul byte found in provided data at position: 2",
    }
    test_unit_from_empty_seq<()> {
        &[
            Token::Seq { len: Some(0) },
            Token::SeqEnd,
        ],
        "invalid type: sequence, expected unit",
    }
    test_unit_from_empty_seq_without_len<()> {
        &[
            Token::Seq { len: None },
            Token::SeqEnd,
        ],
        "invalid type: sequence, expected unit",
    }
    test_unit_from_tuple_struct<()> {
        &[
            Token::TupleStruct { name: "Anything", len: 0 },
            Token::TupleStructEnd,
        ],
        "invalid type: sequence, expected unit",
    }
    test_string_from_unit<String> {
        &[
            Token::Unit,
        ],
        "invalid type: unit value, expected a string",
    }
    test_btreeset_from_unit<BTreeSet<isize>> {
        &[
            Token::Unit,
        ],
        "invalid type: unit value, expected a sequence",
    }
    test_btreeset_from_unit_struct<BTreeSet<isize>> {
        &[
            Token::UnitStruct { name: "Anything" },
        ],
        "invalid type: unit value, expected a sequence",
    }
    test_hashset_from_unit<HashSet<isize>> {
        &[
            Token::Unit,
        ],
        "invalid type: unit value, expected a sequence",
    }
    test_hashset_from_unit_struct<HashSet<isize>> {
        &[
            Token::UnitStruct { name: "Anything" },
        ],
        "invalid type: unit value, expected a sequence",
    }
    test_vec_from_unit<Vec<isize>> {
        &[
            Token::Unit,
        ],
        "invalid type: unit value, expected a sequence",
    }
    test_vec_from_unit_struct<Vec<isize>> {
        &[
            Token::UnitStruct { name: "Anything" },
        ],
        "invalid type: unit value, expected a sequence",
    }
    test_zero_array_from_unit<[isize; 0]> {
        &[
            Token::Unit,
        ],
        "invalid type: unit value, expected an empty array",
    }
    test_zero_array_from_unit_struct<[isize; 0]> {
        &[
            Token::UnitStruct { name: "Anything" },
        ],
        "invalid type: unit value, expected an empty array",
    }
    test_btreemap_from_unit<BTreeMap<isize, isize>> {
        &[
            Token::Unit,
        ],
        "invalid type: unit value, expected a map",
    }
    test_btreemap_from_unit_struct<BTreeMap<isize, isize>> {
        &[
            Token::UnitStruct { name: "Anything" },
        ],
        "invalid type: unit value, expected a map",
    }
    test_hashmap_from_unit<HashMap<isize, isize>> {
        &[
            Token::Unit,
        ],
        "invalid type: unit value, expected a map",
    }
    test_hashmap_from_unit_struct<HashMap<isize, isize>> {
        &[
            Token::UnitStruct { name: "Anything" },
        ],
        "invalid type: unit value, expected a map",
    }
    test_bool_from_string<bool> {
        &[
            Token::Str("false"),
        ],
        "invalid type: string \"false\", expected a boolean",
    }
    test_number_from_string<isize> {
        &[
            Token::Str("1"),
        ],
        "invalid type: string \"1\", expected isize",
    }
    test_integer_from_float<isize> {
        &[
            Token::F32(0.0),
        ],
        "invalid type: floating point `0`, expected isize",
    }
    test_unit_struct_from_seq<UnitStruct> {
        &[
            Token::Seq { len: Some(0) },
            Token::SeqEnd,
        ],
        "invalid type: sequence, expected unit struct UnitStruct",
    }
}

#[derive(Debug, PartialEq)]
struct CompactBinary((u8, u8));

impl<'de> serde::Deserialize<'de> for CompactBinary {
    fn deserialize<D>(deserializer: D) -> Result<CompactBinary, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            <(u8, u8)>::deserialize(deserializer).map(CompactBinary)
        } else {
            <&[u8]>::deserialize(deserializer).map(|bytes| {
                CompactBinary((bytes[0], bytes[1]))
            })
        }
    }
}

#[test]
fn test_human_readable() {
    assert_de_tokens(
        &CompactBinary((1, 2)),
        &[
            Token::Tuple { len: 2},
            Token::U8(1),
            Token::U8(2),
            Token::TupleEnd,
        ],
    );
    assert_de_tokens_readable(
        &CompactBinary((1, 2)),
        &[Token::BorrowedBytes(&[1, 2])],
        false,
    );
}
