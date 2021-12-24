use std::io::Read;

use crate::*;

fn parse(text: &str) -> io::Result<JsonType> {
    let mut string = String::new();
    let mut checker = JsonChecker::new(text.as_bytes());
    checker.read_to_string(&mut string)?;
    let outer_type = checker.finish()?;
    Ok(outer_type)
}

#[test]
fn it_works() {
    let json = r#"{"hello": "girls 😜 ❤️"}"#;

    let mut string = String::new();
    let mut checker = JsonChecker::new(json.as_bytes());
    checker.read_to_string(&mut string).unwrap();
    checker.finish().unwrap();

    assert_eq!(&*string, json);
}

#[test]
fn it_does_not_work() {
    let json = r#"{"hello": "boys}"#; // missing quote

    let mut string = String::new();
    let mut checker = JsonChecker::new(json.as_bytes());
    checker.read_to_string(&mut string).unwrap();

    // This should fail
    checker.finish().unwrap_err();

    assert_eq!(&*string, json);
}

#[test]
fn unclosed_array() {
    assert!(parse(r#"["Unclosed array""#).is_err());
}

#[test]
fn unquoted_key() {
    assert!(parse(r#"{unquoted_key: "keys must be quoted"}"#).is_err());
}

#[test]
fn extra_comma_arr() {
    assert!(parse(r#"["extra comma",]"#).is_err());
}

#[test]
fn double_extra_comma() {
    assert!(parse(r#"["double extra comma",,]"#).is_err());
}

#[test]
fn missing_value() {
    assert!(parse(r#"[   , "<-- missing value"]"#).is_err());
}

#[test]
fn comma_after_close() {
    assert!(parse(r#"["Comma after the close"],"#).is_err());
}

#[test]
fn extra_close() {
    assert!(parse(r#"["Extra close"]]"#).is_err());
}

#[test]
fn extra_comma_obj() {
    assert!(parse(r#"{"Extra comma": true,}"#).is_err());
}

#[test]
fn extra_value_after_close() {
    assert!(parse(r#"{"Extra value after close": true} "misplaced quoted value""#).is_err());
}

#[test]
fn illegal_expression() {
    assert!(parse(r#"{"Illegal expression": 1 + 2}"#).is_err());
}

#[test]
fn illegal_invocation() {
    assert!(parse(r#"{"Illegal invocation": alert()}"#).is_err());
}

#[test]
fn numbers_cannot_have_leading_zeroes() {
    assert!(parse(r#"{"Numbers cannot have leading zeroes": 013}"#).is_err());
}

#[test]
fn numbers_cannot_be_hex() {
    assert!(parse(r#"{"Numbers cannot be hex": 0x14}"#).is_err());
}

#[test]
fn illegal_backslash_escape() {
    assert!(parse(r#"["Illegal backslash escape: \x15"]"#).is_err());
}

#[test]
fn naked() {
    assert!(parse(r#"[\naked]"#).is_err());
}

#[test]
fn illegal_backslash_escape_2() {
    assert!(parse(r#"["Illegal backslash escape: \017"]"#).is_err());
}

#[test]
fn missing_colon() {
    assert!(parse(r#"{"Missing colon" null}"#).is_err());
}

#[test]
fn double_colon() {
    assert!(parse(r#"{"Double colon":: null}"#).is_err());
}

#[test]
fn comma_instead_of_colon() {
    assert!(parse(r#"{"Comma instead of colon", null}"#).is_err());
}

#[test]
fn colon_instead_of_comma() {
    assert!(parse(r#"["Colon instead of comma": false]"#).is_err());
}

#[test]
fn bad_value() {
    assert!(parse(r#"["Bad value", truth]"#).is_err());
}

#[test]
fn single_quote() {
    assert!(parse(r#"['single quote']"#).is_err());
}

#[test]
fn tab_character_in_string() {
    assert!(parse("[\"\ttab\tcharacter\tin\tstring\t\"]").is_err());
}

#[test]
fn tab_character_in_string_esc() {
    assert!(parse("[\"tab\\\tcharacter\\\tin\\\tstring\\\t\"]").is_err());
}

#[test]
fn line_break() {
    assert!(parse("[\"line\nbreak\"]").is_err());
}

#[test]
fn line_break_escaped() {
    assert!(parse("[\"line\\\nbreak\"]").is_err());
}

#[test]
fn no_exponent() {
    assert!(parse(r#"[0e]"#).is_err());
}

#[test]
fn no_exponent_plus() {
    assert!(parse(r#"[0e+]"#).is_err());
}

#[test]
fn exponent_both_signs() {
    assert!(parse(r#"[0e+-1]"#).is_err());
}

#[test]
fn comma_instead_of_closing_brace() {
    assert!(parse(r#"{"Comma instead if closing brace": true,"#).is_err());
}

#[test]
fn missmatch() {
    assert!(parse(r#"["mismatch"}"#).is_err());
}

#[test]
fn unclosed_single_string() {
    assert!(parse(r#""hello"#).is_err())
}

#[test]
fn unfinished_single_boolean() {
    assert!(parse(r#"tru"#).is_err());
    assert!(parse(r#"fa"#).is_err());
}

#[test]
fn unfinished_single_null() {
    assert!(parse(r#"nul"#).is_err());
}

#[test]
fn pass_single_string() {
    assert_eq!(parse(r#""hello""#).unwrap(), JsonType::String);
}

#[test]
fn pass_single_integer() {
    assert_eq!(parse(r#"235896"#).unwrap(), JsonType::Number);
    assert_eq!(parse(r#"-235896"#).unwrap(), JsonType::Number);
}

#[test]
fn pass_single_float() {
    assert_eq!(parse(r#"235896.789076"#).unwrap(), JsonType::Number);
    assert_eq!(parse(r#"-235896.0"#).unwrap(), JsonType::Number);
}

#[test]
fn pass_single_exponent() {
    assert_eq!(parse(r#"235896.10e-56"#).unwrap(), JsonType::Number);
}

#[test]
fn pass_single_fraction() {
    assert_eq!(parse(r#"235896."#).unwrap(), JsonType::Number);
}

#[test]
fn pass_single_boolean() {
    assert_eq!(parse(r#"true"#).unwrap(), JsonType::Bool);
    assert_eq!(parse(r#"false"#).unwrap(), JsonType::Bool);
}

#[test]
fn pass_single_null() {
    assert_eq!(parse(r#"null"#).unwrap(), JsonType::Null);
}

#[test]
fn pass_1() {
    let outer_type = parse(
        r##"

    [
        "JSON Test Pattern pass1",
        {"object with 1 member":["array with 1 element"]},
        {},
        [],
        -42,
        true,
        false,
        null,
        {
            "integer": 1234567890,
            "real": -9876.543210,
            "e": 0.123456789e-12,
            "E": 1.234567890E+34,
            "":  23456789012E66,
            "zero": 0,
            "one": 1,
            "space": " ",
            "quote": "\"",
            "backslash": "\\",
            "controls": "\b\f\n\r\t",
            "slash": "/ & \/",
            "alpha": "abcdefghijklmnopqrstuvwyz",
            "ALPHA": "ABCDEFGHIJKLMNOPQRSTUVWYZ",
            "digit": "0123456789",
            "0123456789": "digit",
            "special": "`1~!@#$%^&*()_+-={':[,]}|;.</>?",
            "hex": "\u0123\u4567\u89AB\uCDEF\uabcd\uef4A",
            "true": true,
            "false": false,
            "null": null,
            "array":[  ],
            "object":{  },
            "address": "50 St. James Street",
            "url": "http://www.JSON.org/",
            "comment": "// /* <!-- --",
            "# -- --> */": " ",
            " s p a c e d " :[1,2 , 3

    ,

    4 , 5        ,          6           ,7        ],"compact":[1,2,3,4,5,6,7],
            "jsontext": "{\"object with 1 member\":[\"array with 1 element\"]}",
            "quotes": "&#34; \u0022 %22 0x22 034 &#x22;",
            "\/\\\"\uCAFE\uBABE\uAB98\uFCDE\ubcda\uef4A\b\f\n\r\t`1~!@#$%^&*()_+-=[]{}|;:',./<>?"
    : "A key can be any string"
        },
        0.5 ,98.6
    ,
    99.44
    ,

    1066,
    1e1,
    0.1e1,
    1e-1,
    1e00,2e+00,2e-00
    ,"rosebud"]

    "##,
    )
    .unwrap();

    assert_eq!(outer_type, JsonType::Array);
}

#[test]
fn pass_2() {
    let outer_type = parse(r#"[[[[[[[[[[[[[[[[[[["Not too deep"]]]]]]]]]]]]]]]]]]]"#).unwrap();

    assert_eq!(outer_type, JsonType::Array);
}

#[test]
fn pass_3() {
    let outer_type = parse(
        r#"

    {
        "JSON Test Pattern pass3": {
            "The outermost value": "must be an object or array.",
            "In this test": "It is an object."
        }
    }

    "#,
    )
    .unwrap();

    assert_eq!(outer_type, JsonType::Object);
}
