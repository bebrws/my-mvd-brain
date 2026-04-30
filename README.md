# Open Source Apache 2.0 memvid CLI and Harness Context/Skills for 3 Harnesses
I have been thinking about building something very similar to this project for a while.
And then found this project memvid that looks to have everything I want.
Except.
There really is vendor lockin. the CLI is not opensource. There arbitrary limitations coded in. Etc.
But everything is Apache 2.0 licenced.
The goal here is to continue the work of creating a something like the o so popular Andrej Karpathy’s LLM Wiki.
What I want and have is tool that automatically records the work I do and, by default, saves what I hope to be pertinent information in a single local file.
The I own. I control. I can backup and move around as I see fit.
Local simple RAG backed by one file.
And all open source.
Thanks to the amazing team that created memvid ( https://github.com/memvid/memvid )

## CLI
In this repo I have created a CLI. You can build the CLI, named `mvd` as to not interfere if you want to use the `memvid` CLI. 
There are also repositories for how this could be used with promeinent harnesses.
Antigravity, Cursor, Claude Code, Codex...

## Why?
memvid is amazing. The creators did amazing work. But I do not want to spend time working with a tool that is *partially* open source. And hinders my ability to work with the datastore I have placed my memory in.
*hidden limits hardcoded* is not great.

## Warning
This CLI was created with the prompt to create the CLI solely from the list of command line arguments from the original `memvid` CLI. 
I have made changes myself. Such as removing hardcoded limits and in some branches modified how large the mv2 capacity would be.
I have reviewed as much of the code as I do to feel safe using this. That there is no telemetry or hidden calls to some 3rd party API.
But again, this CLI was literally created with a single prompt. It is a work in progress. However, it does work well.

## The Original memvid Project

Can be found here: https://github.com/memvid/memvid

## Installing for Antigravity, Claude Code, Codex, or Cursor

How I use it:
Ask the harness to run the following command to install this tool for you so that it runs globally, for each session.

Currently by default it will look for a file named `m2v.json` in your home directory.
So before this will work you will need to build the CLI. Which you can also ask your favorite coding harness to do. And then be sure to place the `mvd` binary into your PATH. Something like /usr/local/bin maybe.
Then run 
```
mvd create $HOME/mvd.mv2
```

And after asking you favorite coding harness to install the files from `mvd-$(your-coding-harness)` so that this will be run for every session and should be placed in the global settings for that harness. You should now be good to go.

## Testing locally

Prompt at some point:
```
Using mvd, what have I tried to implement for this project? And what were the results?
```



