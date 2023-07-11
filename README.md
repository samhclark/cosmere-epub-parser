# Cosmere Search Web Server

This a deployable web server that allows you to perform full text searches on some of Brandon Sanderson's books in the Cosmere. 
Right now, this is only the Era 2 Mistborn books (minus _The Lost Metal_).

There is a `Justfile` you can use to build, lint, run, etc, but it's mostly there to document those commands for me since I don't use Rust that often and always forget.

## Project goals

I wanted to have a full-text search website for Brandon Sanderson's Cosmere, inspired by https://asearchoficeandfire.com. 
But, I really wanted to minimize the hosting costs since it's just a small side project.
Beyond that, I wanted the actual site itself to be very light weight and mobile-friendly.
(I have problems with A Search of Ice and Fire when accessing it over slow connections and also on Firefox for Android.)

In the past, I tried to get it working with off the shelf tools, like ElasticSearch.
I could never really get ES to run with extremely little RAM, like is available on free or cheap VPSs. 
So, that inspired the following goals:

- To minimize latency, the search index should be on the same machine as the webserver
- To minimize cost, the whole system should run with a single core and 256 MB of RAM
- To have excellent search, the index should use stemming and a proper ranking function like BM25
- To be light on the client, the site should have minimal JavaScript (currently non)

Given all those constraints, I was left with either [Tantivy](https://github.com/quickwit-oss/tantivy) (which means Rust) or [Bleve](https://github.com/blevesearch/bleve) (which means Go).
I had been looking for more reasons to use Rust lately, so I went with Tantivy.

## How it works

Using [another tool I wrote](https://github.com/samhclark/cosmere_epub_parser), I parse the EPUB files and turn them into a big doc of newline separated JSON. 
Each paragraph gets a line in that resulting file, and I add some extra metadata about each one. 
This program reads that file at startup and loads it into Tantivy's index. 
This work is done every time at startup, but it's pretty quick and startup time isn't an issue for me. 
Once it's running, that index never changes. 

I use askama as a simple templating library to send HTML back to the client on each request.
So there is no other frontend project laying around -- this is the whole thing. 

All in all that means this is a pretty simple, if disorganized, project. 

## Future work

I track some of the future work as GitHub Issues.
But in broad strokes, I finally purchased "Zero to Production in Rust" so I want to work through that book and come back to this project. 
Hopefully, with some better organizational techniques and with a better testing story.