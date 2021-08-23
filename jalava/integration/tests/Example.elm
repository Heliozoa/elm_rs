module Example exposing (..)

import Expect exposing (Expectation)
import Generated
import Json.Decode
import Test exposing (..)


t : Test
t =
    test "decodes string" <|
        \_ ->
            let
                json =
                    "{\"f1\": \"hello\", \"f2\": \"hallo\"}"
            in
            Expect.equal (Json.Decode.decodeString Generated.sDecoder json) (Ok (Generated.S "hello" "hallo"))
