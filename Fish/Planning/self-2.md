## Self-Evaluation Form for Milestone 2

A fundamental guideline of Fundamentals I, II, and OOD is to design
methods and functions systematically, starting with a signature, a
clear purpose statement (possibly illustrated with examples), and
unit tests.

Under each of the following elements below, indicate below where your
TAs can find:

- the data description of tiles, including an interpretation:
    - https://github.ccs.neu.edu/CS4500-F20/atlanta/blob/91580ce0411a012b0289c7d4049ad17f015f1451/Fish/Common/src/common/tile.rs#L4-L7

      This explains that our tiles are represented as nodes in a graph structure. As such each has
      an adjacency list of tiles it is adjacent to. We call these its "neighbors" and there are 6,
      one for each hexagon side. We also explain each tile has a unique id that it can be identified by.

- the data description of boards, include an interpretation:
    - https://github.ccs.neu.edu/CS4500-F20/atlanta/blob/91580ce0411a012b0289c7d4049ad17f015f1451/Fish/Common/src/common/board.rs#L1-L45

      This one is a bit longer but starts with the description of the board containing a vector of tiles
      and gives more detail on the layout of each tile within the board and how each tile id is assigned
      within line 23. One thing we neglected to mention that we should include in future versions is that
      the board itself is a mapping from each unique tile id to the tile itself and thus does not care
      about the layout of the tiles. This is because our board structure resembles more of a graph, with
      each tile being node with an adjacency list.

- the functionality for removing a tile:
  - purpose:
    - https://github.ccs.neu.edu/CS4500-F20/atlanta/blob/91580ce0411a012b0289c7d4049ad17f015f1451/Fish/Common/src/common/board.rs#L118-L119
  
  - signature:
    - https://github.ccs.neu.edu/CS4500-F20/atlanta/blob/91580ce0411a012b0289c7d4049ad17f015f1451/Fish/Common/src/common/board.rs#L120
  
  - unit tests:
    - https://github.ccs.neu.edu/CS4500-F20/atlanta/blob/91580ce0411a012b0289c7d4049ad17f015f1451/Fish/Common/src/common/board.rs#L230-L252

- the functiinality for reaching other tiles on the board:
  - purpose:
    - https://github.ccs.neu.edu/CS4500-F20/atlanta/blob/91580ce0411a012b0289c7d4049ad17f015f1451/Fish/Common/src/common/tile.rs#L112-L113
  
  - signature:
    - https://github.ccs.neu.edu/CS4500-F20/atlanta/blob/91580ce0411a012b0289c7d4049ad17f015f1451/Fish/Common/src/common/tile.rs#L114
  
  - unit tests:
    - https://github.ccs.neu.edu/CS4500-F20/atlanta/blob/91580ce0411a012b0289c7d4049ad17f015f1451/Fish/Common/src/common/tile.rs#L241-L258

The ideal feedback is a GitHub perma-link to the range of lines in specific
file or a collection of files for each of the above bullet points.

  WARNING: all such links must point to your commit "91580ce0411a012b0289c7d4049ad17f015f1451".
  Any bad links will result in a zero score for this self-evaluation.
  Here is an example link:
    <https://github.ccs.neu.edu/CS4500-F20/atlanta/tree/91580ce0411a012b0289c7d4049ad17f015f1451/Fish>

A lesser alternative is to specify paths to files and, if files are
longer than a laptop screen, positions within files are appropriate
responses.

In either case you may wish to, beneath each snippet of code you
indicate, add a line or two of commentary that explains how you think
the specified code snippets answers the request.
