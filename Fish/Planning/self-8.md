## Self-Evaluation Form for Milestone 8

Indicate below where your TAs can find the following elements in your strategy and/or player-interface modules:

1. did you organize the main function/method for the manager around
the 3 parts of its specifications --- point to the main function

Main function (`run_tournament`): https://github.ccs.neu.edu/CS4500-F20/muleshoe/blob/3e26431d7bfde8a872b285d269679e97cb691bf6/Fish/Common/src/server/manager.rs#L55-L91

We organize the code based around running the tournament and then sending the results of the tournament to the players after the tournament
finishes. We missed the case where the tournament should notify players when the tournament began, as we incorrectly assumed that the referee
sending the initial game state to the players was sufficient.


2. did you factor out a function/method for informing players about
the beginning and the end of the tournament? Does this function catch
players that fail to communicate? --- point to the respective pieces

Informing players about the end of the game: https://github.ccs.neu.edu/CS4500-F20/muleshoe/blob/3e26431d7bfde8a872b285d269679e97cb691bf6/Fish/Common/src/server/manager.rs#L93-L117

Catching players that fail to communicate: https://github.ccs.neu.edu/CS4500-F20/muleshoe/blob/3e26431d7bfde8a872b285d269679e97cb691bf6/Fish/Common/src/server/manager.rs#L110-L113

As stated in point 1, our tournament manager does not directly message players that the tournament has begun.

3. did you factor out the main loop for running the (possibly 10s of
thousands of) games until the tournament is over? --- point to this
function.

Running games until the tournament ends: https://github.ccs.neu.edu/CS4500-F20/muleshoe/blob/3e26431d7bfde8a872b285d269679e97cb691bf6/Fish/Common/src/server/manager.rs#L119-L132


**Please use GitHub perma-links to the range of lines in specific
file or a collection of files for each of the above bullet points.**


  WARNING: all perma-links must point to your commit "3e26431d7bfde8a872b285d269679e97cb691bf6".
  Any bad links will be penalized.
  Here is an example link:
    <https://github.ccs.neu.edu/CS4500-F20/muleshoe/tree/3e26431d7bfde8a872b285d269679e97cb691bf6/Fish>

