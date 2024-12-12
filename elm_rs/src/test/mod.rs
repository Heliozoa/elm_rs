use crate::{Elm, ElmDecode, ElmEncode, ElmQuery, ElmQueryField};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    fmt::Debug,
    io::Write,
    process::{Command, Stdio},
};

mod complex;
mod enums_adjacent;
mod enums_external;
mod enums_internal;
mod enums_untagged;
mod etc_serde;
mod hygiene;
mod nested;
mod query;
mod regression;
mod structs;
mod structs_serde;
mod types;

fn test_json<T: Elm + ElmEncode + ElmDecode + Serialize + DeserializeOwned + PartialEq + Debug>(
    t: T,
) {
    let t_2 = test_json_without_eq(&t, "");
    assert_eq!(t, t_2);
    return;
}

fn test_json_with_deps<
    T: Elm + ElmEncode + ElmDecode + Serialize + DeserializeOwned + PartialEq + Debug,
>(
    t: T,
    deps: &str,
) {
    let t_2 = test_json_without_eq(&t, deps);
    assert_eq!(t, t_2);
    return;
}

fn test_json_without_eq<
    T: Elm + ElmEncode + ElmDecode + ElmDecode + Serialize + DeserializeOwned + Debug,
>(
    t: &T,
    deps: &str,
) -> T {
    let json = serde_json::to_string(&t).unwrap().replace("\"", "\\\"");
    test_with_json(&json, deps)
}

fn test_with_json<T: Elm + ElmEncode + ElmDecode + Serialize + DeserializeOwned + Debug>(
    json: &str,
    deps: &str,
) -> T {
    let encoder_type = T::encoder_type();
    let decoder_type = T::decoder_type();
    let elm_type = T::elm_definition().unwrap();
    let encoder = T::encoder_definition().unwrap();
    let decoder = T::decoder_definition().unwrap();

    let input = format!(
        r#"
import Json.Decode
import Json.Encode
import Dict exposing (Dict)


{}

{}

{}

{}

{}

{}

decoded = Json.Decode.decodeString {} "{}"

reEncoded = Result.map {} decoded

s = case reEncoded of
    Ok value ->
        Json.Encode.encode 0 value
    Err err ->
        Json.Decode.errorToString err

"START"
s
"END"

:exit
"#,
        deps,
        Result::<(), ()>::decoder_definition().unwrap(),
        Result::<(), ()>::encoder_definition().unwrap(),
        elm_type,
        encoder,
        decoder,
        decoder_type,
        json,
        encoder_type
    );

    let json = run_repl(&input);
    let unescaped = unescape::unescape(&json).unwrap();
    println!("{}", unescaped);
    return serde_json::from_str(&unescaped).unwrap();
}

fn test_query<
    T: Elm + ElmEncode + ElmDecode + ElmQuery + Serialize,
    U: Elm + ElmEncode + ElmDecode + ElmQueryField + Serialize,
>(
    val: T,
    expected: &str,
) {
    let json = serde_json::to_string(&val).unwrap().replace("\"", "\\\"");

    let decoder_type = T::decoder_type();
    let elm_type = T::elm_definition().unwrap();
    let query = T::elm_query();
    let query_function = format!("urlEncode{}", T::elm_type());
    let decoder = T::decoder_definition().unwrap();

    let u_elm_type = U::elm_definition().unwrap();
    let u_decoder = U::decoder_definition().unwrap();
    let u_query = U::query_field_encoder_definition().unwrap();

    let input = format!(
        r#"
import Json.Decode
import Url.Builder

{u_elm_type}

{elm_type}

{u_decoder}

{decoder}

{u_query}

{query}

decoded = Json.Decode.decodeString {decoder_type} "{json}"


s = case decoded of
    Ok value ->
        Url.Builder.toQuery ({query_function} value)
    Err err ->
        Json.Decode.errorToString err

"START"
s
"END"

:exit
"#,
    );
    let output = run_repl(&input);
    assert_eq!(output, expected);
}

fn run_repl(input: &str) -> String {
    println!("{}", input);
    let mut cmd = Command::new("elm")
        .arg("repl")
        .current_dir("../elm-test")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let mut stdin = cmd.stdin.take().unwrap();
    stdin.write_all(input.as_bytes()).unwrap();

    let output = cmd.wait_with_output().unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    println!("stdout: {}", stdout);
    println!("stderr: {}", stderr);

    assert!(output.status.success());

    let mut reading = false;
    for line in stdout.lines() {
        if line.contains("END") {
            break;
        }
        if reading {
            let first_quote = line.find('"').unwrap();
            let last_quote = line.rfind('"').unwrap();
            return line[first_quote + 1..last_quote].to_string();
        }
        if line.contains("START") {
            reading = true;
        }
    }
    panic!("not found");
}

#[cfg(any(not(feature = "derive"), not(feature = "serde")))]
compile_error!("The tests require the `derive` and `serde` features to be activated.");
