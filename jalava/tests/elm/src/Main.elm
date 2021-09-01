module Main exposing (..)

import Browser
import Dict
import FormBindings exposing (..)
import Html exposing (text)
import Http



-- MAIN


main : Program () Model Msg
main =
    Browser.element
        { init = init
        , update = \_ model -> ( model, Cmd.none )
        , subscriptions = \_ -> Sub.none
        , view = \_ -> text "none"
        }



-- MODEL


type Model
    = Empty


init : () -> ( Model, Cmd Msg )
init _ =
    ( Empty
    , Cmd.batch
        [ Http.post
            { url = "http://127.0.0.1:8001/a"
            , body = bodyA
            , expect = Http.expectWhatever (\_ -> None)
            }
        , Http.post
            { url = "http://127.0.0.1:8001/b"
            , body = bodyB
            , expect = Http.expectWhatever (\_ -> None)
            }
        , Http.post
            { url = "http://127.0.0.1:8001/c"
            , body = bodyC
            , expect = Http.expectWhatever (\_ -> None)
            }
        , Http.post
            { url = "http://127.0.0.1:8001/d"
            , body = bodyD
            , expect = Http.expectWhatever (\_ -> None)
            }
        ]
    )



-- UPDATE


type Msg
    = None



-- BODIES


bodyA : Http.Body
bodyA =
    prepareA
        { string = "a"
        , s = "b"
        , b = True
        , pu8 = 1
        , pu16 = 2
        , pu32 = 3
        , pu64 = 4
        , pu128 = 5
        , pusize = 6
        , pi8 = 7
        , pi16 = 8
        , pi32 = 9
        , pi64 = 10
        , pi128 = 11
        , pisize = 12
        , nu8 = 13
        , nu16 = 14
        , nu32 = 15
        , nu64 = 16
        , nu128 = 17
        , nusize = 18
        , ni8 = 19
        , ni16 = 20
        , ni32 = 21
        , ni64 = 22
        , ni128 = 23
        , nisize = 24
        , pf32 = 1.1
        , pf64 = 2.2
        }


b : B
b =
    { s = "a"
    , ss = [ "b", "c" ]
    , sm = Dict.fromList [ ( "d", [ "e", "f" ] ) ]
    }


bodyB : Http.Body
bodyB =
    prepareB b


c : C
c =
    { b = b
    , bs = [ b, b ]
    , bm = Dict.fromList [ ( "b", [ b, b, b ] ) ]
    }


bodyC : Http.Body
bodyC =
    prepareC c


d : D
d =
    { c = c
    }


bodyD : Http.Body
bodyD =
    prepareD d
