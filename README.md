# HLS Parsing

## HLS

> HTTP Live Streaming (also known as HLS) is an [HTTP][http]-based [adaptive
> bitrate streaming][abr] communications protocol developed by [Apple
> Inc.][apple] and released in 2009. Support for the protocol is widespread in
> media players, web browsers, mobile devices, and streaming media servers. As
> of 2019, an annual video industry survey has consistently found it to be the
> most popular streaming format.
>
> -- <cite>[Wikipedia][wiki]</cite>

HLS is also what the majority of Disney streaming video products are built on
top of. This protocol breaks down the stream into a series of small media files
which are accessible via HTTP. These files are downloaded sequentially and
played in order to stream the entire presentation. This protocol is defined for
players using [ext-m3u][m3u] format. This file, which has an .m3u8 extension
(the 8 indicating that the file is utf-8 encoded), defines the locations of all
of the media files that need to be downloaded as well as metadata about the
stream. A specification for this format is available [here][spec].

## Challenge

The task for this project is to implement an API which can parse an m3u8 file
to pass the test suite. We are providing a sample m3u8 file as a reference for
you. [This is the infamous (in the streaming world, at least) Big Buck
Bunny][big_buck_bunny].

### Requirements

- Implement the parsing API to read m3u8 files and **pass
  the test suite** (and any new tests you may write!).
    - Note the tests use `Duration::from_secs_f32` to parse media segment durations!
- You do not need to parse any extra data not required for the test suite.
- You do not need to validate the entire m3u8 rigorously, but you should
  provide appropriate error handling for the data required for the test suite.
- Please do not use an existing HLS parsing library.
- It is acceptable to use other external crates and resources.
    - If referencing external code, add a comment with context, i.e., the site,
      the motivation, etc.
- Feel free to move/rearrange/create files and change the public API.
- The code will be evaluated based on correctness and readibility.

Please use pull requests to propose your work.

Please do not spend more than 4 hours on the project.

[abr]: https://en.wikipedia.org/wiki/Adaptive_bitrate_streaming
[apple]: https://en.wikipedia.org/wiki/Apple_Inc.
[big_buck_bunny]: https://docs.evostream.com/sample_content/assets/hls-bunny-rangerequest/bunny/playlist.m3u8
[http]: https://en.wikipedia.org/wiki/HTTP
[m3u]: https://en.wikipedia.org/wiki/M3U#Extended_M3U
[spec]: https://datatracker.ietf.org/doc/html/rfc8216#section-4
[wiki]: https://en.wikipedia.org/wiki/HTTP_Live_Streaming



Implementation: (Sutti: 05/28/2024)
--------------
Steps:

0) Create a main.sr and implement "Hello World!" 
    Ensure there is no error when running "cargo build" and "cargo run"

1) Implement the function 'pub fn parse_ext_m3u(_file: &str)' and return the MediaPlaylist 
    Idea:
      - Create variables.
      - Assign the pointer to the input URL content, In this example the URL content is HLS manifest of Bigbuck_bunney
      - Skip the first line and assume it is the #EXTM3U, if not, stop with "The wrong HLS manifest format"
      - Set the get_url flag to off
      - LOOP with "match" commands or 'if' commands (This code is useing 'match')
        - if the 'get_url' flag is on, read next lines to get the URL of the segment, and add vec[duration, url] to MediaPlayList.segments 

        - Match with "EXT-X-TARGETDURATION": Save target_duration 
        - Match with "EXT-X-VERSION": Save the version to Playlist.
        - Match with "EXTINF": Get the duration of segment, and set the "get_url" flag on
        - Match with "EXT-X-DISCONTINUITY" or "EXT-X-ENDLIST": 
            - Clone the video segment from 'MediaPlaylist.segments' to the discontinutity segments. The starting poing of "segment cloning" should be from the previous discontinutity tag found until the end of the segment vectors. If this is the firts time finding the discontinuity tag, then start the cloning from the begining until the end. 
            - Sum the segments durations of cloning segments.  
            - Add all information of discontinuity(discontinuity duration, and segments vector) to the MedeaPlaylist.discontinuity structure. 
            - If it is the "EXT-X-ENDLIST", also set the 'ended' to ture.
            - Reset the related variables.

      - When th loop is completed:
         - Put the target_duration and version along with segment Vector into the return MediaPlaylist

Output of MediaPlaylist after prasing:
-------------------------------------
MediaPlaylist=[ ended=true, [[duration,url],[duration,url],...,[duration,url]], targetduration=20, version=4, discontinuity=[[total duration, [[duration,url],...,[duration,url]],...,[total duration, [[duration, url],..,[duration,url]]]]


DISCONTINUITY information in the MediaPlaylist:
discontinuity=[
[discontinuity_duration=[secs=25, nanos= 458],
discontinuity_segments=[
[[secs=12,nanos=166],[segment_1440468394459_1440468394459_1.ts]],
[[secs=13,nanos=292],[segment_1440468394459_1440468394459_2.ts]]
],
[discontinuity_duration=[secs=33, nanos= 1376],
discontinuity_segments=[
[[secs=10,nanos=500],[segment_1440468394459_1440468394459_3.ts]],
[[secs=11,nanos=417],[segment_1440468394459_1440468394459_4.ts]],
[[secs=12,nanos=459],[segment_1440468394459_1440468394459_5.ts]]
],
[discontinuity_duration=[secs=40, nanos= 1126],
discontinuity_segments=[
[[secs=14,nanos=000],[segment_1440468394459_1440468394459_6.ts]],
[[secs=19,nanos=292],[segment_1440468394459_1440468394459_7.ts]],
[[secs=7,nanos=834],[segment_1440468394459_1440468394459_8.ts]]
]


Run the code:
-------------
1) Run the test
   - From the terminal of VScode, cd to directory of the project, and then type
    "cargo test"
    Note: There is no error and warning. 
Expectaton:
All test cases must PASS

2) Build
   - From the terminal of VScode, cd to directory of the project, and then type
   "cargo build"

3) Run: 
Note: This is not an application project, there is nothing to showcase oter than "Hello World!"....No surprises. 
   - From the terminal of VScode, cd to directory of the project, and then type
   "cargo run"

4) Clean:
   - From the terminal of VScode, cd to directory of the project, and then type
   "cargo clean"


For example:
-----------
   - From the terminal of VScode, cd to directory of the project, and then type
    cargo clean; cargo build; cargo test

Screenshot
----------
PS C:\Users\ssuki\disney_hls_parser> cargo clean; cargo build; cargo run; cargo test
   Compiling anyhow v1.0.86
   Compiling disney_hls_parser v0.1.0 (C:\Users\ssuki\disney_hls_parser)
    Finished dev [unoptimized + debuginfo] target(s) in 1.55s
    Finished dev [unoptimized + debuginfo] target(s) in 0.01s
     Running `target\debug\disney_hls_parser.exe`
Hello, world!
   Compiling indoc v2.0.5
   Compiling disney_hls_parser v0.1.0 (C:\Users\ssuki\disney_hls_parser)
    Finished test [unoptimized + debuginfo] target(s) in 0.81s
     Running unittests src\lib.rs (target\debug\deps\disney_hls_parser-584febc3da365396.exe)

running 7 tests
test media_playlist::tests::big_buck_bunny::parses_discontinuity ... ok
test media_playlist::tests::big_buck_bunny::parses_end_tag ... ok
test media_playlist::tests::big_buck_bunny::parses_segments ... ok
test media_playlist::tests::big_buck_bunny::parses_target_duration ... ok
test media_playlist::tests::big_buck_bunny::parses_version ... ok
test media_playlist::tests::big_buck_bunny::parses_target_duration_negative_ts ... ok
test media_playlist::tests::big_buck_bunny::parses_version_negative_ts ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src\main.rs (target\debug\deps\disney_hls_parser-7e42c32160fd89f0.exe)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests disney_hls_parser

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

PS C:\Users\ssuki\disney_hls_parser>

Limitations (know issues)
-------------------------
1) The project is NOT designed to validate the HLS manifest tags. It only focus of some M3U tags, as menstioned above.
2) If the code attempts to parse digits from an input string containing alphabetic characters, it will panic with helpful error messages.
3) (More can be added here)
.
.

Development Tools
-----------------
- VScode and cargo.
- RUST and RUST extension on VScode.
- git tools. 
- Internet access to github.(optional)