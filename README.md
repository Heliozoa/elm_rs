Automatically generate type definitions and functions for your Elm frontend from your Rust backend types. Currently supports generating
- Elm types with the `Elm` trait and derive macro
- JSON encoders and decoders, compatible with `serde_json`, with the `ElmJson` trait and derive macro
- Multipart form requests that can be parsed by Rocket's `FromForm` with the `ElmForm` and `ElmFormParts` traits and derive macros
Note that attributes that are used to configure `serde_json` or `#[derive(FromForm)]` are not taken into account yet.

For example, the following Rust types
```rust
use jalava::{Elm, ElmJson, ElmForm};

#[derive(Elm, ElmJson, ElmForm)]
enum Filetype {
    Jpeg,
    Png,
}

#[derive(Elm, ElmJson, ElmForm)]
struct Drawing {
    title: String,
    authors: Vec<String>,
    filename: String,
    filetype: Filetype,
}
```
allow us to generate the following Elm code
```elm
type Filetype
    = Jpeg
    | Png


filetypeDecoder : Json.Decode.Decoder Filetype
filetypeDecoder =
    Json.Decode.oneOf
        [ Json.Decode.andThen
            (\x ->
                case x of
                    "Jpeg" ->
                        Json.Decode.succeed Jpeg

                    _ ->
                        Json.Decode.fail "invalid enum variant"
            )
            Json.Decode.string
        , Json.Decode.andThen
            (\x ->
                case x of
                    "Png" ->
                        Json.Decode.succeed Png

                    _ ->
                        Json.Decode.fail "invalid enum variant"
            )
            Json.Decode.string
        ]


filetypeEncoder : Filetype -> Json.Encode.Value
filetypeEncoder enum =
    case enum of
        Jpeg ->
            Json.Encode.string "Jpeg"

        Png ->
            Json.Encode.string "Png"


filetypeToString : Filetype -> String
filetypeToString enum =
    case enum of
        Jpeg ->
            "Jpeg"

        Png ->
            "Png"


type alias Drawing =
    { title : String
    , authors : List String
    , filename : String
    , filetype : Filetype
    }


drawingDecoder : Json.Decode.Decoder Drawing
drawingDecoder =
    Json.Decode.succeed Drawing
        |> Json.Decode.andThen (\x -> Json.Decode.map x (Json.Decode.field "title" Json.Decode.string))
        |> Json.Decode.andThen (\x -> Json.Decode.map x (Json.Decode.field "authors" (Json.Decode.list Json.Decode.string)))
        |> Json.Decode.andThen (\x -> Json.Decode.map x (Json.Decode.field "filename" Json.Decode.string))
        |> Json.Decode.andThen (\x -> Json.Decode.map x (Json.Decode.field "filetype" filetypeDecoder))


drawingEncoder : Drawing -> Json.Encode.Value
drawingEncoder struct =
    Json.Encode.object
        [ ( "title", Json.Encode.string struct.title )
        , ( "authors", Json.Encode.list Json.Encode.string struct.authors )
        , ( "filename", Json.Encode.string struct.filename )
        , ( "filetype", filetypeEncoder struct.filetype )
        ]


prepareDrawing : Drawing -> Http.Body
prepareDrawing form =
    Http.multipartBody <|
        List.concat
            [ [ Http.stringPart "title" (identity form.title) ]
            , List.concat (List.concat (List.indexedMap (\i0 x0 -> [ [ Http.stringPart "authors[" ++ String.fromInt i0 ++ "]" (identity x0) ] ]) (identity form.authors)))
            , [ Http.stringPart "filename" (identity form.filename) ]
            , [ Http.stringPart "filetype" (filetypeToString form.filetype) ]
            ]
```

### 0.1.0
- [x] Generate Elm types with the `Elm` trait and derive macro
- [x] Generate JSON encoders and decoders with the `ElmJson` trait and derive macro
- [x] Generate Elm functions that create multipart requests compatible with Rocket's multipart form parsing through the `rocket::{ElmForm, ElmFormField}` traits and derive macros

### Planned
- [ ] Compatibility with serde attributes (e.g. `rename`)
- [ ] Compatibility with rocket attributes (e.g. `field`)
- [ ] Optionally include definitions for the dependencies of exported types
- [ ] Implement support for as many `serde::{Deserialize, Serialize}` std types as possible
  - [ ] IpAddr, Ipv4Addr, Ipv6Addr
  - [ ] SocketAddr, SocketAddrV4, SocketAddrV6
  - [ ] PhantomData
- [ ] Handle recursive types
- [ ] Handle generic types
- [ ] Improve generated code

### License
Licensed under either one of
- Apache License, Version 2.0
- The MIT License
