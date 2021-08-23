module BindingsTest exposing (..)

import Bindings exposing (..)
import Dict exposing (Dict)
import Expect
import Json.Decode exposing (decodeString)
import Json.Encode exposing (..)
import Test exposing (..)


suite : Test
suite =
    describe "Ensure the generated bindings work properly"
        [ describe "Unit" <|
            endecoderTests """null""" Unit unitEncoder unitDecoder
        , describe "Newtype" <|
            endecoderTests """true""" (Newtype True) newtypeEncoder newtypeDecoder
        , describe "Tuple" <|
            endecoderTests """[
  123,
  [
    [
      1.1,
      2.2
    ],
    null
  ]
]""" (Tuple 123 [ Just [ 1.1, 2.2 ], Nothing ]) tupleEncoder tupleDecoder
        , describe "Record" <|
            endecoderTests """{
  "borrow": true,
  "one_tuple": [
    true
  ],
  "two_tuple": [
    true,
    false
  ],
  "three_tuple": [
    true,
    false,
    true
  ],
  "mut_borrow": false,
  "arc": true,
  "abool": false,
  "au8": 0,
  "au16": 1,
  "au32": 2,
  "au64": 3,
  "ausize": 4,
  "ai8": 5,
  "ai16": 6,
  "ai32": 7,
  "ai64": 8,
  "aisize": 9,
  "bmap": {
    "a": true
  },
  "bset": [
    true
  ],
  "b": true,
  "cell": false,
  "cow": "cow",
  "duration": {
    "secs": 1,
    "nanos": 123
  },
  "map": {
    "b": false
  },
  "set": [
    false
  ],
  "list": [
    true,
    false
  ],
  "mutex": true,
  "nu8": 10,
  "nu16": 11,
  "nu32": 12,
  "nu64": 13,
  "nu128": 14,
  "nusize": 15,
  "ni8": 16,
  "ni16": 17,
  "ni32": 18,
  "ni64": 19,
  "ni128": 20,
  "nisize": 21,
  "some": false,
  "none": null,
  "path": "path",
  "pathbuf": "pathbuf",
  "rc": true,
  "refcell": false,
  "result_ok": {
    "Ok": true
  },
  "result_err": {
    "Err": false
  },
  "rwlock": true,
  "string": "string",
  "system_time": {
    "secs_since_epoch": 0,
    "nanos_since_epoch": 0
  },
  "vec": [
    false,
    true
  ],
  "slice": [
    false,
    true
  ],
  "array": [
    true,
    false
  ],
  "pu8": 22,
  "pu16": 23,
  "pu32": 24,
  "pu64": 25,
  "pu128": 26,
  "pusize": 27,
  "pi8": 28,
  "pi16": 29,
  "pi32": 30,
  "pi64": 31,
  "pi128": 32,
  "pisize": 33,
  "pf32": 34.349998474121094,
  "pf64": 36.37,
  "ss": "str",
  "uuid": "be81c148-3ebe-4e0b-949a-e4a706f4dbde",
  "nt": "11:22:33",
  "nd": "2020-10-01",
  "ndt": "2021-08-23T21:47:20.000001234",
  "dt": "2021-08-23T21:47:20.000001234Z"
}"""
                { borrow = True
                , one_tuple = True
                , two_tuple = ( True, False )
                , three_tuple = ( True, False, True )
                , mut_borrow = False
                , arc = True
                , abool = False
                , au8 = 0
                , au16 = 1
                , au32 = 2
                , au64 = 3
                , ausize = 4
                , ai8 = 5
                , ai16 = 6
                , ai32 = 7
                , ai64 = 8
                , aisize = 9
                , bmap = Dict.fromList [ ( "a", True ) ]
                , bset = [ True ]
                , b = True
                , cell = False
                , cow = "cow"
                , duration = { secs = 1, nanos = 123 }
                , map = Dict.fromList [ ( "b", False ) ]
                , set = [ False ]
                , list = [ True, False ]
                , mutex = True
                , nu8 = 10
                , nu16 = 11
                , nu32 = 12
                , nu64 = 13
                , nu128 = 14
                , nusize = 15
                , ni8 = 16
                , ni16 = 17
                , ni32 = 18
                , ni64 = 19
                , ni128 = 20
                , nisize = 21
                , some = Just False
                , none = Nothing
                , path = "path"
                , pathbuf = "pathbuf"
                , rc = True
                , refcell = False
                , result_ok = Ok True
                , result_err = Err False
                , rwlock = True
                , string = "string"
                , system_time =
                    { secs_since_epoch = 0, nanos_since_epoch = 0 }
                , vec = [ False, True ]
                , slice = [ False, True ]
                , array = [ True, False ]
                , pu8 = 22
                , pu16 = 23
                , pu32 = 24
                , pu64 = 25
                , pu128 = 26
                , pusize = 27
                , pi8 = 28
                , pi16 = 29
                , pi32 = 30
                , pi64 = 31
                , pi128 = 32
                , pisize = 33
                , pf32 = 34.349998474121094
                , pf64 = 36.37
                , ss = "str"
                , uuid = "be81c148-3ebe-4e0b-949a-e4a706f4dbde"
                , nt = "11:22:33"
                , nd = "2020-10-01"
                , ndt = "2021-08-23T21:47:20.000001234"
                , dt = "2021-08-23T21:47:20.000001234Z"
                }
                recordEncoder
                recordDecoder
        , describe "CustomType::V1" <|
            endecoderTests "\"V1\"" V1 customTypeEncoder customTypeDecoder
        , describe "CustomType::V2" <|
            endecoderTests """{
  "V2": null
}""" (V2 Unit) customTypeEncoder customTypeDecoder
        , describe "CustomType::V3" <|
            endecoderTests """{
  "V3": [
    true,
    [
      123,
      [
        [
          1.1,
          2.2
        ],
        null
      ]
    ]
  ]
}""" (V3 (Newtype True) (Tuple 123 [ Just [ 1.1, 2.2 ], Nothing ])) customTypeEncoder customTypeDecoder
        , describe "CustomType::V4" <|
            endecoderTests """{
  "V4": {
    "r": null
  }
}"""
                (V4
                    { r = Unit }
                )
                customTypeEncoder
                customTypeDecoder
        ]


endecoderTests : String -> a -> (a -> Value) -> Json.Decode.Decoder a -> List Test
endecoderTests json elm encoder decoder =
    [ test "decodes" <|
        \_ ->
            Expect.equal (Ok elm) (decodeString decoder json)
    , test "encodes" <|
        \_ ->
            Expect.equal json (encode 2 (encoder elm))
    ]
