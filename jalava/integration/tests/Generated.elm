module Generated exposing (..)

import Json.Decode
import Json.Decode.Pipeline


type alias S =
    { f1 : String
    , f2 : String
    }


sDecoder : Json.Decode.Decoder S
sDecoder =
    Json.Decode.succeed S
        |> Json.Decode.Pipeline.required "f1" Json.Decode.string
        |> Json.Decode.Pipeline.required "f2" Json.Decode.string
