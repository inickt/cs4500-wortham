## Self-Evaluation Form for Milestone 1

### General 

We will run self-evaluations for each milestone this semester.  The
graders will evaluate them for accuracy and completeness.

Every self-evaluation will go out into your Enterprise GitHub repo
within a short time after the milestone deadline, and you will have 24
hours to answer the questions and push back a completed form.

This one is a practice run to make sure you get


### Specifics 


- does your analysis cover the following ideas:

  - the need for an explicit Interface specification between the (remote) AI 
    players and the game system?

    - We specify that the client and server communicates via messages, though we leave the format of message unspecified (from “Game Server” list item 1 in system.pdf):

    > Accepts client messages, validates them, updates the gamestate accordingly, and sends the clients the new game state.

  - the need for a referee sub-system for managing individual games

    - Our “Game Server” acts as referee and takes care of managing individual games, from the 3rd sentence of our first paragraph in system.pdf:

    > The first is the game server, which is responsible for managing a single game. This component will include the business logic for the game

  - the need for a tournament management sub-system for grouping
    players into games and dispatching to referee components

    - Our “Tournament Server” is a tournament management system. From the last sentence in the opening paragraph in system.pdf:

    >  the tournament server will contain the organization business logic for tournaments: setting up/organizing game servers and communicating with them to determine winners

    - The “Tournament Server” heading in system.pdf further specifies the tournament server is responsible for allocating players to each game and starting those games:

    > Handle player-sign up [...] scheduling each game on the game server with the participating players

- does your building plan identify concrete milestones with demo prototypes:

  - for running individual games

    - Milestone 2 specifies that the game can now be played to completion both locally and over the network. Milestone 2 in milestone.pdf:

    > The game can now be played to completion and allows clients to connect over the network to play. This demo should show that the game is capable of being played in our highly-distributed world.

  - for running complete tournaments on a single computer 

    - The first sentence in the “Milestone 4” heading in milestone.pdf specifies tournaments can now be “organized and played by multiple players.” Since our milestones are cumulative this includes both the local players and online players specified to work in milestone 1/2.

  - for running remote tournaments on a network

    - The first sentence in the “Milestone 4” heading in milestone.pdf specifies tournaments can now be “organized and played by multiple players.” Since our milestones are cumulative this includes both the local players and online players specified to work in milestone 1/2.


- for the English of your memo, you may wish to check the following:

  - is each paragraph dedicated to a single topic? does it come with a
    thesis statement that specifies the topic?

    - Each paragraph either has a topic sentence that defines the topic; or, they are organized into headings and bulleted lists. See our first sentence in the first paragraph of milestone.pdf for an example:

    > In this memo, we provide a description of the software system we plan to build for the Fish game.



  - do sentences make a point? do they run on?

    - Each sentence has a purpose not served by any other sentence. See the first paragraph of system.pdf for an example of this; the sentences successively describe the purposes of different parts of our system, each of which are expanded in subsequent heading/list pairs. Our sentences are also generally quite short and do not run on.


  - do sentences connect via old words/new words so that readers keep
    reading?

    - Our sentences are strung together via transition words, which are not reused within paragraphs. See the first paragraph of system.pdf, in which we use the words “first”, “next”, and “finally” to switch the discussion between different parts of our system.


  - are all sentences complete? Are they missing verbs? Objects? Other
    essential words?

    - Each sentence is complete. Some sentences under bulleted lists do not provide subjects, because the subject is implied given the underlined header above each bullet. In our experience, this is conventional in bulleted lists. See below for an example from the “Game Client” heading in system.pdf:

    > Accepts gamestate messages from the server showing the current game state. These may be rendered by a gui to show the game to human players in a more appealing form.


  - did you make sure that the spelling is correct? ("It's" is *not* a
    possesive; it's short for "it is". "There" is different from
    "their", a word that is too popular for your generation.)

    - We did use correct spelling, and we checked our spelling using the Google Docs spell-checker before turning in our memo.




The ideal feedback are pointers to specific senetences in your memo.
For PDF, the paragraph/sentence number suffices. 

For **code repos**, we will expect GitHub line-specific links.
