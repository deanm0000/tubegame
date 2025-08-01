You know that simple game on phones where you've got a bunch of test tubes with colorful liquid in them and the object is to get each color in just one test tube. Well there was one in particular that I was absolutely sure was unsolvable and they just wanted people to spend money for a cheat. Well I took that itch combined with my itch to learn rust and I made a clone of the game so that I could then make a brute force solver.

The game has two modes, one is getting a random puzzle and the other is getting the specific puzzle that seemed to be unsolveable. My solver can not solve the fixed puzzle but it solves most random puzzles in under a few seconds so I believe I'm vindicated in thinking it's unsolveable. 

In random mode you can pick how many colors (they're really just letters in the CLI) you want (up to 12) and how many empty vials there should be. The colors per vial is hard coded at 4.

When you're in the game you type the tube number you want to take from, a space, and then the tube number you want to pour that into. There's no undo feature and playing via CLI is pretty awful so I wouldn't expect anyone to want to play this outright. That said, my main objective was just learning by doing and getting the solver so those things are big successes in my book.

At this point I just wanted to put this out incase it can help anybody (not that I'm an expert to emulate). Criticism welcome too.