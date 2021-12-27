use crate::{Elm, ElmJson};
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
mod structs;
mod structs_serde;
mod types;
mod rocket;
mod hygiene;

fn test<T: Elm + ElmJson + Serialize + DeserializeOwned + PartialEq + Debug>(t: T) {
    let t_2 = test_without_eq(&t, "");
    assert_eq!(t, t_2);
    return;
}

fn test_with_deps<T: Elm + ElmJson + Serialize + DeserializeOwned + PartialEq + Debug>(
    t: T,
    deps: &str,
) {
    let t_2 = test_without_eq(&t, deps);
    assert_eq!(t, t_2);
    return;
}

fn test_without_eq<T: Elm + ElmJson + Serialize + DeserializeOwned + Debug>(
    t: &T,
    deps: &str,
) -> T {
    let json = serde_json::to_string(&t).unwrap().replace("\"", "\\\"");
    test_with_json(&json, deps)
}

fn test_with_json<T: Elm + ElmJson + Serialize + DeserializeOwned + Debug>(
    json: &str,
    deps: &str,
) -> T {
    let encoder_type = T::encoder_type();
    let decoder_type = T::decoder_type();
    let elm_type = T::elm_definition().unwrap();
    let encoder = T::encoder_definition().unwrap();
    let decoder = T::decoder_definition().unwrap();
    println!("{}", elm_type);
    println!("{}", encoder);
    println!("{}", decoder);

    let mut cmd = Command::new("elm")
        .arg("repl")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let mut stdin = cmd.stdin.take().unwrap();
    writeln!(
        stdin,
        r#"
import Json.Decode
import Json.Encode
import Dict exposing (Dict)

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
        deps, elm_type, encoder, decoder, decoder_type, json, encoder_type
    )
    .unwrap();

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
            let json = &line[first_quote + 1..last_quote];
            let unescaped = unescape::unescape(json).unwrap();
            println!("{}", unescaped);
            return serde_json::from_str(&unescaped).unwrap();
        }
        if line.contains("START") {
            reading = true;
        }
    }
    panic!("not found");
}
