use std::time::Duration;

use media_remote::prelude::*;

fn main() {
    // Create an instance of NowPlaying to interact with the media remote.
    let now_playing = NowPlayingJXA::new(Duration::from_secs(30));

    // Use a guard lock to safely access media information within this block.
    // The guard should be released as soon as possible to avoid blocking.
    {
        let guard = now_playing.get_info();
        let info = guard.as_ref();

        // If information is available, print the title of the currently playing media.
        if let Some(info) = info {
            println!("Currently playing: {:?}", info);
        }
    }

    // Toggle the play/pause state of the media.
    // now_playing.toggle();
}
