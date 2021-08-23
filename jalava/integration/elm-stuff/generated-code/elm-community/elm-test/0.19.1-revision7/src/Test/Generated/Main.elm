module Test.Generated.Main exposing (main)

import Example
import Generated

import Test.Reporter.Reporter exposing (Report(..))
import Console.Text exposing (UseColor(..))
import Test.Runner.Node
import Test

main : Test.Runner.Node.TestProgram
main =
    Test.Runner.Node.run
        { runs = 100
        , report = ConsoleReport UseColor
        , seed = 179689513486213
        , processes = 16
        , globs =
            []
        , paths =
            [ "/drive/barracuda-ext4/Dev/jalava/integration/tests/Example.elm"
            , "/drive/barracuda-ext4/Dev/jalava/integration/tests/Generated.elm"
            ]
        }
        [ ( "Example"
          , [ Test.Runner.Node.check Example.t
            ]
          )
        , ( "Generated"
          , [ Test.Runner.Node.check Generated.sDecoder
            ]
          )
        ]