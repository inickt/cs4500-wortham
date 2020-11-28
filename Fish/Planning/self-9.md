## Self-Evaluation Form for Milestone 9

You must make an appointment with your grader during his or her office
hour to demo your project. See the end of the self-eval for the assigned
grader. 

Indicate below where your TA can find the following elements in your strategy 
and/or player-interface modules: 

1. for human players, point the TA to
   - the interface (signature) that an AI player implements  
     https://github.ccs.neu.edu/CS4500-F20/wortham/blob/995396fa77a9022124466be37d5fa2453d9dc789/Fish/Common/src/client/player.rs#L7-L20
   
   - the interface that the human-GUI component implements  
     https://github.ccs.neu.edu/CS4500-F20/wortham/blob/995396fa77a9022124466be37d5fa2453d9dc789/Fish/Common/src/server/connection.rs#L16-L19
     
     both types of players are represented by a ClientProxy enumeration [here](https://github.ccs.neu.edu/CS4500-F20/wortham/blob/64a12ffaf06cf78ece094f1411420ff4ad4ded3e/Fish/Common/src/server/serverclient.rs#L30-L37)
   
   - the implementation of the player GUI  
     Our gamestate printing logic is [here](https://github.ccs.neu.edu/CS4500-F20/wortham/blob/995396fa77a9022124466be37d5fa2453d9dc789/Fish/Common/src/common/gamestate.rs#L68-L115)
     and the player gamestate printing and input loop is [here](https://github.ccs.neu.edu/CS4500-F20/wortham/blob/995396fa77a9022124466be37d5fa2453d9dc789/9/Other/src/main.rs#L33-L45).
   

2. for game observers, point the TA to
   - the `game-observer` interface that observers implement 
   - the point where the `referee` consumes observers 
   - the callback from `referee` to observers concerning turns

3. for tournament observers, point the TA to
   - the `tournament-observer` interface that observers implement 
   - the point where the `manager` consumes observers 
   - the callback to observes concerning the results of rounds 


Do not forget to meet the assigned TA for a demo; see bottom.  If the
TA's office hour overlaps with other obligations, sign up for a 1-1.


**Please use GitHub perma-links to the range of lines in specific
file or a collection of files for each of the above bullet points.**


  WARNING: all perma-links must point to your commit "995396fa77a9022124466be37d5fa2453d9dc789".
  Any bad links will be penalized.
  Here is an example link:
    <https://github.ccs.neu.edu/CS4500-F20/wortham/tree/995396fa77a9022124466be37d5fa2453d9dc789/Fish>

Assigned grader = Evan Hiroshige (hiroshige.e@northeastern.edu)

