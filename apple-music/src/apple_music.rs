use crate::application_data::ApplicationData;
use crate::error::Error;
use crate::playlist::Playlist;
use crate::script_controller::{ParamType, ScriptController};
use crate::track::Track;
use strum_macros::Display;

/// Strict entry point of the module containing the whole logic.
pub struct AppleMusic(String);

impl Default for AppleMusic {
    fn default() -> Self {
        Self("Music".to_string())
    }
}

impl AppleMusic {
    /// New with specified app name
    pub fn new(app: String) -> Self {
        Self(app)
    }

    /// Returns an up-to-date ApplicationData struct.
    pub fn get_application_data(&self) -> Result<ApplicationData, Error> {
        match ScriptController.execute_script::<ApplicationData>(
            ParamType::ApplicationData,
            None,
            None,
        ) {
            Ok(data) => Ok(data),
            Err(err) => Err(err),
        }
    }

    /// Looks for and returns a Playlist based on provided id, if it exists.
    pub fn get_playlist_by_id(&self, id: i32) -> Result<Playlist, Error> {
        match ScriptController.execute_script::<Playlist>(ParamType::PlaylistById, Some(id), None) {
            Ok(data) => Ok(data),
            Err(err) => Err(err),
        }
    }

    /// Returns currently playing Track, if any.
    pub fn get_current_track(&self) -> Result<Track, Error> {
        match ScriptController.execute_script::<Track>(ParamType::CurrentTrack, None, None) {
            Ok(data) => Ok(data),
            Err(err) => Err(err),
        }
    }

    /// Fetches and returns a list of all Library Tracks.
    /// WARNING: Might fail if more than 900 Tracks are to be returned, due to a JavaScript limit.
    pub fn get_all_library_tracks(&self) -> Result<Vec<Track>, Error> {
        match ScriptController.execute_script::<Vec<Track>>(ParamType::AllTracks, None, None) {
            Ok(data) => Ok(data),
            Err(err) => Err(err),
        }
    }

    /// Plays the provided Track on AppleMusic player.
    pub fn play_track(&self, track: &Track) -> Result<(), Error> {
        let cmd = format!(
            "Application('{}').play(Application('{}').tracks.byId({}))",
            self.0, self.0, track.id
        );

        let _ = ScriptController.execute(cmd.as_str(), None);

        Ok(())
    }

    /// Plays the provided Playlist on AppleMusic player.
    pub fn play_playlist(&self, playlist: &Playlist) -> Result<(), Error> {
        let cmd = format!(
            "Application('{}').play(Application('{}').playlists.byId({}))",
            self.0, self.0, playlist.id
        );

        let _ = ScriptController.execute(cmd.as_str(), None);

        Ok(())
    }

    /// Mutes / Unmutes AppleMusic player.
    pub fn set_mute(&self, value: bool) -> Result<(), Error> {
        let cmd = format!("Application('{}').mute = {}", self.0, value);

        let _ = ScriptController.execute(cmd.as_str(), None);

        Ok(())
    }

    /// Activates / Deactivates Shuffle mode on AppleMusic player.
    pub fn set_shuffle(&self, value: bool) -> Result<(), Error> {
        let cmd = format!("Application('{}').shuffleEnabled = {}", self.0, value);

        let _ = ScriptController.execute(cmd.as_str(), None);

        Ok(())
    }

    /// Sets Song Repeat mode to provided value.
    pub fn set_song_repeat_mode(&self, value: SongRepeatMode) -> Result<(), Error> {
        let cmd = format!(
            "Application('{}').songRepeat = \"{}\"",
            self.0,
            value.to_string()
        );

        let _ = ScriptController.execute(cmd.as_str(), None);

        Ok(())
    }

    /// Sets Sound Volume to provided value. ( 0 <= value <= 100 )
    pub fn set_sound_volume(&self, value: i8) -> Result<(), Error> {
        let cmd = format!("Application('{}').soundVolume = {}", self.0, value);

        let _ = ScriptController.execute(cmd.as_str(), None);

        Ok(())
    }

    /// Tries to convert the provided Track.
    pub fn convert_track(&self, track: &Track) -> Result<(), Error> {
        let cmd = format!(
            "Application('{}').convert(Application('{}').tracks.byId({}))",
            self.0, self.0, track.id
        );

        let _ = ScriptController.execute(cmd.as_str(), None);

        Ok(())
    }

    /// Resumes the player if a track is Paused, otherwise Plays a Track from Library.
    pub fn play(&self) -> Result<(), Error> {
        let cmd = format!("Application('{}').play()", self.0);

        let _ = ScriptController.execute(&cmd, None);

        Ok(())
    }

    /// Pauses the player's currently playing Track.
    pub fn pause(&self) -> Result<(), Error> {
        let cmd = format!("Application('{}').pause()", self.0);

        let _ = ScriptController.execute(&cmd, None);

        Ok(())
    }

    /// Stops Rewinding / Fast-Forwarding and plays the Track at normal speed.
    pub fn resume(&self) -> Result<(), Error> {
        let cmd = format!("Application('{}').resume()", self.0);

        let _ = ScriptController.execute(&cmd, None);

        Ok(())
    }

    /// Restart the current Track.
    pub fn back_track(&self) -> Result<(), Error> {
        let cmd = format!("Application('{}').backTrack()", self.0);

        let _ = ScriptController.execute(&cmd, None);

        Ok(())
    }

    /// Fast-forwards the current Track up until resuming or end of current Track.
    pub fn fast_forward(&self) -> Result<(), Error> {
        let cmd = format!("Application('{}').fastForward()", self.0);

        let _ = ScriptController.execute(&cmd, None);

        Ok(())
    }

    /// Skips current Track and plays next one.
    pub fn next_track(&self) -> Result<(), Error> {
        let cmd = format!("Application('{}').nextTrack()", self.0);

        let _ = ScriptController.execute(&cmd, None);

        Ok(())
    }

    /// Plays if Player is currently Paused, Pauses if Player is currently Playing.
    pub fn playpause(&self) -> Result<(), Error> {
        let cmd = format!("Application('{}').playpause()", self.0);

        let _ = ScriptController.execute(&cmd, None);

        Ok(())
    }

    /// Goes back to previous Track and plays it.
    pub fn previous_track(&self) -> Result<(), Error> {
        let cmd = format!("Application('{}').previousTrack()", self.0);

        let _ = ScriptController.execute(&cmd, None);

        Ok(())
    }

    /// Rewinds current Track up until resuming or start of Track.
    pub fn rewind(&self) -> Result<(), Error> {
        let cmd = format!("Application('{}').rewind()", self.0);

        let _ = ScriptController.execute(&cmd, None);

        Ok(())
    }

    /// Stops player, removing enqueued Tracks and currently playing Track.
    pub fn stop(&self) -> Result<(), Error> {
        let cmd = format!("Application('{}').stop()", self.0);

        let _ = ScriptController.execute(&cmd, None);

        Ok(())
    }

    /// Fully Quits Apple Music.
    pub fn quit(&self) -> Result<(), Error> {
        let cmd = format!("Application('{}').quit()", self.0);

        let _ = ScriptController.execute(&cmd, None);

        Ok(())
    }

    /// Opens Apple Music app.
    pub fn run(&self) -> Result<(), Error> {
        let cmd = format!("Application('{}').run()", self.0);

        let _ = ScriptController.execute(&cmd, None);

        Ok(())
    }
}

/// Currently playing Repeat mode
#[derive(Display)]
#[strum(serialize_all = "lowercase")]
pub enum SongRepeatMode {
    OFF,
    ONE,
    ALL,
}
