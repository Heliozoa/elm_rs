Automatically generate type definitions and JSON encoders and JSON decoders for your Elm frontend from your Rust backend types.

For example, the following Rust types
```rust
use jalava::Elm;

#[derive(Elm)]
enum Filetype {
    Jpeg,
    Png,
}

#[derive(Elm)]
struct Drawing {
    title: String,
    authors: Vec<String>,
    filename: String,
    filetype: Filetype,
}
```
would result in the following Elm code
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


type alias Drawing =
    { title : String
    , authors : List (String)
    , filename : String
    , filetype : Filetype
    }


drawingDecoder : Json.Decode.Decoder Drawing
drawingDecoder =
    Json.Decode.succeed Drawing
        |> Json.Decode.andThen (\x -> Json.Decode.map x (Json.Decode.field "title" (Json.Decode.string)))
        |> Json.Decode.andThen (\x -> Json.Decode.map x (Json.Decode.field "authors" (Json.Decode.list (Json.Decode.string))))
        |> Json.Decode.andThen (\x -> Json.Decode.map x (Json.Decode.field "filename" (Json.Decode.string)))
        |> Json.Decode.andThen (\x -> Json.Decode.map x (Json.Decode.field "filetype" (filetypeDecoder)))



drawingEncoder : Drawing -> Json.Encode.Value
drawingEncoder struct =
    Json.Encode.object
        [ ( "title", (Json.Encode.string) struct.title )
        , ( "authors", (Json.Encode.list (Json.Encode.string)) struct.authors )
        , ( "filename", (Json.Encode.string) struct.filename )
        , ( "filetype", (filetypeEncoder) struct.filetype )
        ]
```

### 0.1.0 Roadmap
- [x] Implement `Elm` trait and derive macro
- [ ] Implement `ElmForm` and `ElmFormField` traits and derive macros to generate functions for requests using `Http.multipartBody`

### 0.1.1+ Roadmap
- [ ] Implement `Elm` for all std types supported by `serde::{Deserialize, Serialize}` where possible
- [ ] Include definitions for the dependencies of exported types
- [ ] Compatibility with serde attributes (e.g. `rename`)
- [ ] Handle recursive data structures

### License
Licensed under either one of
- Apache License, Version 2.0
- The MIT License
