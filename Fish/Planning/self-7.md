## Self-Evaluation Form for Milestone 7

Please respond to the following items with

1. the item in your `todo` file that addresses the points below.
    It is possible that you had "perfect" data definitions/interpretations
    (purpose statement, unit tests, etc) and/or responded to feedback in a 
    timely manner. In that case, explain why you didn't have to add this to
    your `todo` list.

2. a link to a git commit (or set of commits) and/or git diffs the resolve
   bugs/implement rewrites: 

These questions are taken from the rubric and represent some of the most
critical elements of the project, though by no means all of them.

(No, not even your sw arch. delivers perfect code.)

### Board

- a data definition and an interpretation for the game _board_
  - For documentation & interpretation of the Board as a whole, We had
  gotten this feedback in a prior milestone but elected to fix it then, so
  it is not included now. The commit
  https://github.ccs.neu.edu/CS4500-F20/muleshoe/commit/0f2c0a938b553498377970af27d988aaea30edeb
  added most of this documentation.
  - We did elect to add more documentation for how holes are represented
  this milestone.
    - TODO item: https://github.ccs.neu.edu/CS4500-F20/muleshoe/commit/57d054d0cdea4b1a88024f439f6e22674200c7dd#diff-fffda276c12b3dbca7ad810a9a8364e5R17-R18

    - Resolved in: https://github.ccs.neu.edu/CS4500-F20/muleshoe/commit/5a1a221775800857bc5656bd1c9848d8586eab7a#diff-ef17ff4b28d86c0a6510697c3e53ad37R27-R31

- a purpose statement for the "reachable tiles" functionality on the board representation
  - Our purpose statement for all_reachable_tiles was already present and good enough to
  never have gotten negative feedback on it. We didn't add any more to it because we felt
  it already explained what the function did well enough.
  - https://github.ccs.neu.edu/CS4500-F20/muleshoe/blob/91bea8104db14cf5e973cc9dbcf728b10170a511/Fish/Common/src/common/tile.rs#L115-L117

- two unit tests for the "reachable tiles" functionality
  - We had one unit test beforehand, and added another unit test this milestone to address
    past feedback.
  - TODO item that addresses this: 
  https://github.ccs.neu.edu/CS4500-F20/muleshoe/blame/master/7/todo.md#L6
  - The commit that adds this test is:
  https://github.ccs.neu.edu/CS4500-F20/muleshoe/blob/91bea8104db14cf5e973cc9dbcf728b10170a511/Fish/Common/src/common/tile.rs#L274-L297

### Game States 


- a data definition and an interpretation for the game _state_
  - We've had this feedback in the past but have fixed it already
  in prior milestones. Here is the commit we added the documentation
  in: https://github.ccs.neu.edu/CS4500-F20/muleshoe/blob/95791f0a918f2ad4e00f265044b7997c23f9fa0b/Fish/Common/src/common/gamestate.rs#L36-L57
  - diff for above commit: https://github.ccs.neu.edu/CS4500-F20/muleshoe/commit/95791f0a918f2ad4e00f265044b7997c23f9fa0b#diff-19440ccde3b1d99acf68680483f4ab1dR54-R56

- a purpose statement for the "take turn" functionality on states
  - We do not have a todo item for this because we already documented the advance_turn
    function's effects and interpretation in the last milestone.
  - https://github.ccs.neu.edu/CS4500-F20/muleshoe/blame/master/Fish/Common/src/common/gamestate.rs#L263-L268

- two unit tests for the "take turn" functionality
  - We have one unit test for `advance_turn`. This test handles advancing turns for each
  player turn until a full round has passed. Then it asserts that it is the first player's
  turn again.
  - TODO item that addresses this: https://github.ccs.neu.edu/CS4500-F20/muleshoe/blame/master/7/todo.md#L10
  - The commit that adds this test: 
  https://github.ccs.neu.edu/CS4500-F20/muleshoe/blob/3444536cf6243635922c85fce66a99f311966555/Fish/Common/src/common/gamestate.rs#L469-L481


### Trees and Strategies


- a data definition including an interpretation for _tree_ that represent entire games
  - We added a sentence to clarify how a game tree can be used to plan multiple moves ahead (and by extension represent entire games).
  - TODO item that addresses this: https://github.ccs.neu.edu/CS4500-F20/muleshoe/blame/master/7/todo.md#L15
  - The commit that adds this clarification: 
  https://github.ccs.neu.edu/CS4500-F20/muleshoe/blame/3444536cf6243635922c85fce66a99f311966555/Fish/Planning/games.md#L26-L27

- a purpose statement for the "maximin strategy" functionality on trees
  - We have no todo item for this: Our maximin move strategy has detailed
  documentation split across the 3 functions that implement it.
  - The main/wrapper function has the general usage and strategy of the function:
  https://github.ccs.neu.edu/CS4500-F20/muleshoe/blame/master/Fish/Common/src/client/strategy.rs#L64-L69
  - The recursive function used within specifies how it traverses the game tree and its
  termination conditions: https://github.ccs.neu.edu/CS4500-F20/muleshoe/blame/master/Fish/Common/src/client/strategy.rs#L76-L87
  - And the tie breaker function documents what it does to break ties: https://github.ccs.neu.edu/CS4500-F20/muleshoe/blame/master/Fish/Common/src/client/strategy.rs#L111-L117

- two unit tests for the "maximin" functionality 
  - We have no todo item for this. We already have two "maximin" unit tests located at: 
  https://github.ccs.neu.edu/CS4500-F20/muleshoe/blame/master/Fish/Common/src/client/strategy.rs#L205-L290


### General Issues

Point to at least two of the following three points of remediation: 

- the replacement of `null` for the representation of holes with an actual representation 
  - We have no TODO item for this. We use rust which does not have null as a language feature
  (outside of unsafe functions like std::mem::zeroed). We elected to use an Option type from
  the beginning.

- one name refactoring that replaces a misleading name with a self-explanatory name
  - For this milestone, we did not have to refactor anything regarding names.
  - We received feedback in the past that our "place_penguin_zigzag" name was confusing,
  so we changed it last milestone, splitting it into "find_zigzag_placement" and
  "take_zigzag_placement": https://github.ccs.neu.edu/CS4500-F20/muleshoe/commit/7d4fa6c5089053f6cae0852d68244adb86439ac3#diff-b8c8c3ab57cb97dbe5361ea28a458ca1R24-R33

- a "debugging session" starting from a failed integration test:
  - the failed integration test
  - its translation into a unit test (or several unit tests)
  - its fix
  - bonus: deriving additional unit tests from the initial ones 
    - We fixed any feedback from failing integration tests in prior milestones before milestone 7.
    - We did have a problem with the Milestone 4's integration test during this milestone,
    but it ended up being a problem with the testing harness itself and was not a bug
    in the codebase so it did not result in a unit test. The problem was when we added
    the "occupied_tiles" functionality and unit tests after milestone 4, we did not go
    back to test and fix milestone 4's integration test until recently.
    
### Bonus

Explain your favorite "debt removal" action via a paragraph with
supporting evidence (i.e. citations to git commit links, todo, `bug.md`
and/or `reworked.md`).

  - Our favorite "debt removal" action was adding a unit test for ensuring that player's
  turns are advancing correctly. Before this milestone, we did not have a unit test at all
  for this functionality. This was potentially dangerous for such a core function of the game.
  If we ever changed/refactored the way turns are represented or changed in any way we would have
  had no test to ensure that change is correct. Resultingly, we decided to make a robust unit test
  that can test that all turns can advance correctly, and then go back to the first player's turn
  after advancing from the last player.

  - Item in Todo.md: https://github.ccs.neu.edu/CS4500-F20/muleshoe/commit/5a1a221775800857bc5656bd1c9848d8586eab7a#diff-fffda276c12b3dbca7ad810a9a8364e5R9

  - Added unit test: https://github.ccs.neu.edu/CS4500-F20/muleshoe/commit/c90f30f7a5985e81c241771c1e2475b90beaec03#diff-19440ccde3b1d99acf68680483f4ab1dR468-R480