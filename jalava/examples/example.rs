#![allow(dead_code)]

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

fn main() {
    let elm_types = format!(
        "
{}

{}

{}

{}

{}


{}",
        Filetype::elm_definition().unwrap(),
        Filetype::decoder_definition().unwrap(),
        Filetype::encoder_definition().unwrap(),
        Drawing::elm_definition().unwrap(),
        Drawing::decoder_definition().unwrap(),
        Drawing::encoder_definition().unwrap(),
    );
    println!("{}", elm_types);
    assert_eq!(
        elm_types,
        r#"
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
"#
    )
}
