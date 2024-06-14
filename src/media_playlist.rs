// //! Utilites for parsing media playlists (i.e. not master playlists).
#![allow(unused)]

use core::time::Duration;
use anyhow::{anyhow, Result};
use std::num::ParseIntError;

/// Storage for HLS Media Playlist data. Can be constructed from `ext-m3u` data using
/// [`parse_ext_m3u`][MediaPlaylist::parse_ext_m3u].
#[derive(Debug, Clone, PartialEq)]
pub struct MediaPlaylist {
    /// Whether or not an ENDLIST tag was found. See
    /// <https://datatracker.ietf.org/doc/html/rfc8216#section-4.3.3.4>.
    ended: bool,

    // [ MediaSegment, MediaSegment, MediaSegment, MediaSegment...]
    // [ [Duration, string], [Duration, string], [Duration, string],...]
    segments: Vec<MediaSegment>,

    /// Duration that no media segment can exceed. See
    /// <https://datatracker.ietf.org/doc/html/rfc8216#section-4.3.3.1>.
    /// This is the value from the #EXT-X-TARGETDURATION tag.
    ///  secs: u64,
    /// nanos: Nanoseconds
    /// Duration:  [secs, nanos]
    target_duration: Duration,

    /// Version of playlist for compatibility. See
    /// <https://datatracker.ietf.org/doc/html/rfc8216#section-4.3.1.2>.
    version: u64,

    // The video segment between the discontinuity tag 
    // [ [[Duration, string], [Duration, string], [Duration, string]...],  
    //   [[Duration, string], [Duration, string], [Duration, string],...], 
    //   [[Duration, string], [Duration, string], [Duration, string],...]
    //  ]
    discontinuity: Vec<DiscontinuitySegment>,
}

/// A media segment contains information to actually load the presentation. See [the
/// specification][spec] for more details.
///
/// [spec]: https://datatracker.ietf.org/doc/html/rfc8216#section-3
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaSegment {
    /// From the #EXTINF tag. See <https://datatracker.ietf.org/doc/html/rfc8216#section-4.3.2.1>.
    ///  secs: u64,
    /// nanos: Nanoseconds
    /// Duration:  [secs, nanos]
    duration: Duration,

    /// Relative URL of media segment. See
    /// <https://datatracker.ietf.org/doc/html/rfc8216#section-4.3.2> and
    /// <https://datatracker.ietf.org/doc/html/rfc8216#section-4.1>.
    url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscontinuitySegment {
    // sum of segment durations before the EXT-X-DISCONTINUITY
    //  secs: u64,
    // nanos: Nanoseconds
    // Duration:  [secs, nanos]
    discontinuity_duration: Duration,

    // segment before the EXT-X-DISCONTINUITY
    discontinuity_segments: Vec<MediaSegment>,
}


impl MediaPlaylist {
    // Parses the given file into a [`MediaPlaylist`], returning an error if the file does not
    // adhere to the specification.
    pub fn parse_ext_m3u(_file: &str) -> Result<Self> {

        //*** Variables for MedisPlaylist Structure ***/
        //set the ended to false
        let mut ended = false;
        // Create a new vector for storing the segments
        // Such ["10.000:a_01.ts", "10.102:a_02.ts", "10:113:a_03.ts"...]
        //
        let mut segments = Vec::new();
        // Create a new variable to store the target duration
        // // set to default value 0
        let mut target_duration = Duration::new(0, 0);

        // Create a new variable to store the version
        let mut version = None;

        // Create a new vector for storing the discontinuity segments
        // Such [[[30.225,["10.000:a_01.ts", "10.102:a_02.ts", "10:113:a_03.ts"], ["10.000:a_01.ts", "10.102:a_02.ts", "10:113:a_03.ts"], ["10.000:a_01.ts", "10.102:a_02.ts", "10:113:a_03.ts"]],
        let mut discontinuity: Vec<_> = Vec::new();

        //*** Valiables for process */
        // Create a new variable to store the lines of the file
        let mut lines = _file.lines();

        // Skip the first line (assumed to be #EXTM3U)
        // .next() means using slide.
        if lines.next().unwrap_or_default() != "#EXTM3U" {
            return Err(anyhow!("Missing #EXTM3U header"));
        }

        // variable to store the duration of the segment
        let mut duration_seg = Duration::from_secs_f32(0.000);

        // Set Variable to store the duration of the discontinuity segment:
        // I need this variable to be on milliseconds because 
        // it will be used for arithmetic operation (sums).
        // The issue with using 'from_secs_f32()' arises when performing summation, 
        // as it may introdure extra digit in nanosecconds
        // potentially causing failure in 'assert_eq!()' statement within the test suite.
        // Ref: https://doc.rust-lang.org/core/time/struct.Duration.html
        let mut sum_discontinuity_duration = Duration::from_millis(0); // use ::from_millis()

        let mut discontinuity_flag = true;

        // Create a new variable to store the flag to get the url of the segment
        let mut get_url = false;

        // start segment index to clone the segments from prious discontinuity tag
        let mut start_discontinuity_segment = 0;

        fn u64_from_string (s: &str) -> Result<u64, String> {
            let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
            match digits.parse::<u64>() {
                Ok(value) => Ok(value),
                Err(_) => Err(String::from("Error: the string contains non-numeric characters")),
            }
        }        

        fn f32_from_string (s: &str) -> Result<f32, String> {
            let digits: String = s.chars().filter(|c| c.is_ascii_digit() || *c == '.').collect();
            match digits.parse::<f32>() {
                Ok(value) => Ok(value),
                Err(_) => Err(String::from("Error: the string contains non-numeric characters")),
            }
        }        

        //get into the LOOP to parse manifest content line by line
        for line in lines {
            if get_url { //found the duration, then looking for url for the segment
                if line.contains(".ts") { //check if the line contains the url
                    // *** Save the duration and url to MediaPlaylist.segments.
                    segments.push(MediaSegment { duration: duration_seg, url: line.to_string() });

                    // *** Save discontinuity
                    // MydiaPlaylist = [...
                    //              [ [Segment_Duration, string], [Segment_Duration, string] ], ...]
                    //                 |
                    // discontinuity = |----> [ [discontinuity_duration,[[Segment_Duration, string],...,[Segment_Duration, string]],...,]
                    if discontinuity.is_empty() || discontinuity_flag { // create a new discontinuity vector and push the segment
                        let mut discontinuity_segment = DiscontinuitySegment {
                            discontinuity_segments: vec![MediaSegment { duration: duration_seg, url: line.to_string() }],  // creating a new vector containing a single 'MeidaSegment' struct
                            discontinuity_duration: duration_seg,
                        };
                        discontinuity.push(discontinuity_segment);
                        discontinuity_flag = false;
                    } else { 
                        // if the discontinuity is not empty, then get the last element of the discontinuity
                        // and push the segment to the last element of the discontinuity, then pump up the duration
                        let last_discontinuity = discontinuity.last_mut().unwrap();
                        // sum the discontinuity duration in milliseconds
                        let sum_discontinuity_duration = last_discontinuity.discontinuity_duration.as_millis() + duration_seg.as_millis() as u128;
                        // Then save back in the Duration format.
                        last_discontinuity.discontinuity_duration = Duration::from_millis(sum_discontinuity_duration.try_into().unwrap());
                        // Then push the segment to the last element of the discontinuity
                        last_discontinuity.discontinuity_segments.push(MediaSegment { duration: duration_seg, url: line.to_string() });
                    }

                    // Set get_url flag OFF
                    get_url = false;
                    continue;
                } else { // if the line does not contain the url, then get the next line, may need to handle the error here if the HLS content is missing ".ts"
                    continue; // skip the line below
                }
            }

            match line.to_string() {
                s if s.contains("EXT-X-TARGETDURATION") => {
                    let target_duration_str = s
                    .split(':')
                    .last()
                    .ok_or_else(|| anyhow!("EXT-X-TARGETDURATION: expecting digit")).unwrap();

                    //Save the target_duration
                    // by using library Duration and from_secs() function
                    // Note: the from_secs will set the nanos to 0.
                    // secs: u64,
                    // nanos: Nanoseconds
                    // Duration:  [secs, nanos]
                    match u64_from_string(target_duration_str) {
                         Ok(num) => target_duration = Duration::from_secs(num),
                         Err(err) => println!{"EXT-X-TARGETDURATION: expecting digit in HLS manifest after 'EXT-X-TARGETDURATION:' tag"},
                    }

                },
                s if s.contains("#EXT-X-VERSION:") => { // HLS manifest version
                    //#EXT-X-VERSION:4
                    // Try with string slice to get a string starting from the length of "EXT-X-VERSION:" until the end of the line
                    // convert the string to u64
                    // If the .parse return an error, the ok() will set the version to None 
                    version = line["#EXT-X-VERSION:".len()..]// get the value after the "#EXT-X-VERSION:"
                        .parse()// convert to u64
                        .ok(); // if error, set to None
                },
                s if s.contains("#EXTINF:") => { // segment duration
                    // // ------parsing to get the durration by using string slice ------
                    // // #EXTINF:12.166,
                    let duration_f32 = line["#EXTINF:".len()..]// string slide to get the value after the "12.166,"
                        .splitn(2,',')// 12.166, => ["12.166", ""]
                        .next().unwrap();// get the first part, "12.166"

                    // Put the duration_f32 in the Duration struct{[secs, nanos]}
                    // by using the from_secs_f32() function because we need to preserve the nanos
                    match f32_from_string(duration_f32) {
                            Ok(num) => duration_seg = Duration::from_secs_f32(num),
                            Err(err) => println!{"EXTINF: expecting digit in HLS manifest after 'EXTINF:' tag"},
                    }
   
                    // need to get the url of the segment in the next two lines, so set get_url to true
                    // turn get_url flag ON
                    get_url = true;
               },
               s if s.contains("#EXT-X-DISCONTINUITY") => { // IF found the EXT-X-DISCONTINUITY tag,
                    // Set discontinuity flag to true
                    discontinuity_flag = true;
                },
                s if s.contains("#EXT-X-ENDLIST") => { // FOUND the end of the playlist
                    // set the ended to true
                    ended = true;
                },
               _ => { // do nothing
                    continue;
                }
            }
        } //end of loop

        // if the version is None, return an error message
        let version = version.ok_or_else(|| anyhow!("Missing #EXT-X-VERSION"))?;

        // return the MediaPlaylist with the values
        // { ended: bool, segments: Vec<MediaSegment>, target_duration: Duration, version: u64}
        // put in Ok() to return the Result<Self>
        Ok(MediaPlaylist { ended, segments, target_duration, version, discontinuity })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod big_buck_bunny {
        use super::*;
        // Helper because this playlist is valid and should parse correctly.
        fn big_buck_bunny() -> MediaPlaylist {
            const BIG_BUCK_BUNNY: &str = indoc::indoc! {"
            #EXTM3U
            #EXT-X-VERSION:4
            #EXT-X-ALLOW-CACHE:NO
            #EXT-X-TARGETDURATION:20
            #EXT-X-MEDIA-SEQUENCE:1
            #EXT-X-PROGRAM-DATE-TIME:2015-08-25T01:59:23.708+00:00
            #EXTINF:12.166,
            #EXT-X-BYTERANGE:1430680@4048392
            segment_1440468394459_1440468394459_1.ts
            #EXTINF:13.292,
            #EXT-X-BYTERANGE:840360@5479072
            segment_1440468394459_1440468394459_2.ts
            #EXT-X-DISCONTINUITY
            #EXTINF:10.500,
            #EXT-X-BYTERANGE:1009184@6319432
            segment_1440468394459_1440468394459_3.ts
            #EXTINF:11.417,
            #EXT-X-BYTERANGE:806332@0
            segment_1440468394459_1440468394459_4.ts
            #EXTINF:12.459,
            #EXT-X-BYTERANGE:701616@806332
            segment_1440468394459_1440468394459_5.ts
            #EXT-X-DISCONTINUITY
            #EXTINF:14.000,
            #EXT-X-BYTERANGE:931352@1507948
            segment_1440468394459_1440468394459_6.ts
            #EXTINF:19.292,
            #EXT-X-BYTERANGE:1593676@2439300
            segment_1440468394459_1440468394459_7.ts
            #EXTINF:7.834,
            #EXT-X-BYTERANGE:657812@4032976
            segment_1440468394459_1440468394459_8.ts
            #EXT-X-ENDLIST
        "};
            MediaPlaylist::parse_ext_m3u(BIG_BUCK_BUNNY).expect("Big Buck Bunny should parse")
        }

        #[test]
        fn parses_version() {
            let playlist = big_buck_bunny();
            assert_eq!(playlist.version, 4);
        }

        #[test]
        fn parses_version_negative_ts() {
            let playlist = big_buck_bunny();
            assert_ne!(playlist.version, 3);
        }

        #[test]
        fn parses_target_duration() {
            let playlist = big_buck_bunny();
            assert_eq!(playlist.target_duration, Duration::from_secs(20));
        }

        #[test]
        fn parses_target_duration_negative_ts() {
            let playlist = big_buck_bunny();
            assert_ne!(playlist.target_duration, Duration::from_secs(21));
        }

        #[test]
        fn parses_end_tag() {
            let playlist = big_buck_bunny();
            assert!(playlist.ended);
        }

         #[test]
        fn parses_segments() {
            let playlist = big_buck_bunny();
            let expected = vec![
                MediaSegment {
                    duration: Duration::from_secs_f32(12.166),
                    url: "segment_1440468394459_1440468394459_1.ts".to_string(),
                },
                MediaSegment {
                    duration: Duration::from_secs_f32(13.292),
                    url: "segment_1440468394459_1440468394459_2.ts".to_string(),
                },
                MediaSegment {
                    duration: Duration::from_secs_f32(10.500),
                    url: "segment_1440468394459_1440468394459_3.ts".to_string(),
                },
                MediaSegment {
                    duration: Duration::from_secs_f32(11.417),
                    url: "segment_1440468394459_1440468394459_4.ts".to_string(),
                },
                MediaSegment {
                    duration: Duration::from_secs_f32(12.459),
                    url: "segment_1440468394459_1440468394459_5.ts".to_string(),
                },
                MediaSegment {
                    duration: Duration::from_secs_f32(14.000),
                    url: "segment_1440468394459_1440468394459_6.ts".to_string(),
                },
                MediaSegment {
                    duration: Duration::from_secs_f32(19.292),
                    url: "segment_1440468394459_1440468394459_7.ts".to_string(),
                },
                MediaSegment {
                    duration: Duration::from_secs_f32(7.834),
                    url: "segment_1440468394459_1440468394459_8.ts".to_string(),
                },
            ];

            // Slightly easier to read failures if we go one at a time.
            assert_eq!(playlist.segments.len(), expected.len());
            //loop through the playlist segments and compare with the expected
            //This code use zip() to iterate over two iterators at the same time
            //and compare the values
            //into_iter() is used to consume the vector and return an iterator of owned values
            //zip() is used to iterate over two iterators at the same time
            //and compare the values
            // playlist.segments = [(duration:[sec,nano], url:string)...]
            // actual   =  (duration:[sec,nano], url:string)
            // expected = [(duration:[sec,nano], url:string)...]
            for (actual, expected_elm) in playlist.segments.into_iter().zip(expected) {
                assert_eq!(actual, expected_elm);
            }
        }

        #[test]
        fn parses_discontinuity() {
            let playlist = big_buck_bunny();
            let expected = vec![
                DiscontinuitySegment {
                    discontinuity_duration: Duration::from_millis(25457),
                    discontinuity_segments: vec![
                        MediaSegment {
                            duration: Duration::from_secs_f32(12.166),
                            url: "segment_1440468394459_1440468394459_1.ts".to_string(),
                        },
                        MediaSegment {
                            duration: Duration::from_secs_f32(13.292),
                            url: "segment_1440468394459_1440468394459_2.ts".to_string(),
                        },
                    ],
                },
                DiscontinuitySegment {
                    discontinuity_duration: Duration::from_millis(34374),
                    discontinuity_segments: vec![
                        MediaSegment {
                            duration: Duration::from_secs_f32(10.500),
                            url: "segment_1440468394459_1440468394459_3.ts".to_string(),
                        },
                        MediaSegment {
                            duration: Duration::from_secs_f32(11.417),
                            url: "segment_1440468394459_1440468394459_4.ts".to_string(),
                        },
                        MediaSegment {
                            duration: Duration::from_secs_f32(12.459),
                            url: "segment_1440468394459_1440468394459_5.ts".to_string(),
                        },
                    ],
                },
                DiscontinuitySegment {
                    discontinuity_duration: Duration::from_millis(41125),
                    discontinuity_segments: vec![
                        MediaSegment {
                            duration: Duration::from_secs_f32(14.000),
                            url: "segment_1440468394459_1440468394459_6.ts".to_string(),
                        },
                        MediaSegment {
                            duration: Duration::from_secs_f32(19.292),
                            url: "segment_1440468394459_1440468394459_7.ts".to_string(),
                        },
                        MediaSegment {
                            duration: Duration::from_secs_f32(7.834),
                            url: "segment_1440468394459_1440468394459_8.ts".to_string(),
                        },
                    ],
                },
            ];
            // loop through the discontinuity segments and compare the discontinuity_duration and
            // get in side the discontinuity segments and compare the duration and url with above value in the expected
            // [ [discontinuity_duration=34.375, [ [Duration, string], [Duration, string], [Duration, string] ] ],
            // [ [discontinuity_duration=25.458, [ [Duration, string], [Duration, string], [Duration, string] ] ],
            // [ [discontinuity_duration=41.126, [ [Duration, string], [Duration, string], [Duration, string] ] ],
            for (outter_actual, outter_expected) in playlist.discontinuity.into_iter().zip(expected) {
                // compare the discontinuity_duration
                assert_eq!(outter_actual.discontinuity_duration, outter_expected.discontinuity_duration);
                // [Duration, string], [Duration, string], [Duration, string]...],
                for (inner_actual, inner_expected) in outter_actual.discontinuity_segments.into_iter().zip(outter_expected.discontinuity_segments) {
                    assert_eq!(inner_actual, inner_expected);
                }
            }
        }
    }
}