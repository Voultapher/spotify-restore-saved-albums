use std::collections::HashSet;
use std::error::Error;

use chrono::Utc;

use rspotify::spotify::client::Spotify;
use rspotify::spotify::model::album::FullAlbum;
use rspotify::spotify::model::track::FullTrack;
use rspotify::spotify::oauth2::{SpotifyClientCredentials, SpotifyOAuth};
use rspotify::spotify::util::get_token;

use indicatif::ProgressBar;

use structopt::StructOpt;

fn read_all_saved_tracks(
    spotify: &Spotify,
) -> Result<Vec<FullTrack>, Box<Error>> {
    let page_size = 50;
    let mut saved_tracks = Vec::new();

    let mut offset = 0;

    let track_read_bar = ProgressBar::new_spinner();

    loop {
        let saved_tracks_chunk =
            spotify.current_user_saved_tracks(page_size, offset)?;
        saved_tracks.extend_from_slice(saved_tracks_chunk.items.as_slice());
        offset += page_size;

        track_read_bar
            .set_message(&format!("Read {} tracks", saved_tracks.len()));
        track_read_bar.inc(1);

        if saved_tracks_chunk.next.is_none() {
            break;
        }
    }

    track_read_bar.finish();

    saved_tracks.sort_by(|a, b| a.added_at.cmp(&b.added_at));

    Ok(saved_tracks.iter().map(|item| item.track.clone()).collect())
}

fn backup_saved_tracks(
    spotify: &Spotify,
    saved_tracks: &[FullTrack],
) -> Result<(), Box<Error>> {
    let user_id = spotify.current_user()?.id;

    let backup_playlist_name = format!("srsa backup {}", Utc::now());

    println!("Creating backup playlist: {}", backup_playlist_name);

    let backup_playlist = spotify.user_playlist_create(
        &user_id,
        &backup_playlist_name,
        Some(false),
        Some("Backup of ALL saved songs created by spotify-restore-saved-albums".into())
    )?;

    let mut offset = 0;

    let backup_bar = ProgressBar::new(saved_tracks.len() as u64);

    for saved_tracks_chunk in saved_tracks.chunks(50) {
        let track_ids = saved_tracks_chunk
            .iter()
            .filter(|track| track.id.is_some())
            .map(|track| track.id.clone().unwrap())
            .collect::<Vec<String>>();

        spotify.user_playlist_add_tracks(
            &user_id,
            &backup_playlist.id,
            track_ids.as_slice(),
            Some(offset),
        )?;

        offset += saved_tracks_chunk.len() as i32;

        backup_bar.inc(saved_tracks_chunk.len() as u64);
    }

    backup_bar.finish();

    Ok(())
}

fn read_all_saved_albums(
    spotify: &Spotify,
) -> Result<Vec<FullAlbum>, Box<Error>> {
    let page_size = 50;
    let mut saved_albums = Vec::new();

    let mut offset = 0;

    let album_read_bar = ProgressBar::new_spinner();

    loop {
        let saved_albums_chunk =
            spotify.current_user_saved_albums(page_size, offset)?;
        offset += saved_albums_chunk.items.len() as u32;

        saved_albums.extend_from_slice(saved_albums_chunk.items.as_slice());

        album_read_bar.set_message(&format!("Read {} albums", offset));
        album_read_bar.inc(1);

        if saved_albums_chunk.next.is_none() {
            break;
        }
    }

    album_read_bar.finish();

    Ok(saved_albums.iter().map(|item| item.album.clone()).collect())
}

fn delete_all_saved_albums(
    spotify: &Spotify,
    saved_albums: &[FullAlbum],
) -> Result<(), Box<Error>> {
    let mut album_ids = saved_albums
        .iter()
        .map(|album| album.id.clone())
        .collect::<Vec<String>>();

    // If spotify would have a sane API this wouldn't be needed.
    album_ids.dedup();

    let album_delete_bar = ProgressBar::new(album_ids.len() as u64);
    println!("Unsaving albums");

    // The API documents a max 50 ids for delete.
    // Yet 50 ids results in 502 :/
    for album_ids_chunk in album_ids.as_slice().chunks(20) {
        spotify.current_user_saved_albums_delete(album_ids_chunk)?;

        album_delete_bar.inc(album_ids_chunk.len() as u64);
    }

    album_delete_bar.finish();

    Ok(())
}

fn save_albums(
    spotify: &Spotify,
    saved_tracks: &[FullTrack],
) -> Result<(), Box<Error>> {
    let mut already_saved_album_ids = HashSet::new();
    let mut album_ids = Vec::new();

    for saved_track in saved_tracks {
        if let Some(album_id) = &saved_track.album.id {
            if already_saved_album_ids.contains(album_id) {
                continue;
            }

            already_saved_album_ids.insert(album_id);
            album_ids.push(album_id.clone());
        }
    }

    let album_save_bar = ProgressBar::new(album_ids.len() as u64);
    println!("Saving albums");

    // Minimize API calls, add albums in batches.
    // The API documents a max 50 ids for save.
    // Yet 50 ids results in 502 :/
    //
    // Save each album individually, because bulk save does not respect id order.
    for album_ids_chunk in album_ids.as_slice().chunks(1) {
        if spotify
            .current_user_saved_albums_add(album_ids_chunk)
            .is_err()
        {
            // Maybe you ran into max library size of 10k, try clean up.
            println!("Library overflowed, cleaning up");
            let spilled_saved_tracks = read_all_saved_tracks(&spotify)?;
            delete_spilled_saved_tracks(
                &spotify,
                saved_tracks,
                spilled_saved_tracks.as_slice(),
            )?;

            println!("Saving albums");
            spotify.current_user_saved_albums_add(album_ids_chunk)?;
        }
        album_save_bar.inc(album_ids_chunk.len() as u64);
    }

    album_save_bar.finish();

    Ok(())
}

fn delete_spilled_saved_tracks(
    spotify: &Spotify,
    original_saved_tracks: &[FullTrack],
    spilled_saved_tracks: &[FullTrack],
) -> Result<(), Box<Error>> {
    let mut original_track_ids = HashSet::new();

    for original_saved_track in original_saved_tracks {
        if let Some(track_id) = &original_saved_track.id {
            original_track_ids.insert(track_id);
        }
    }

    let spilled_track_ids = spilled_saved_tracks
        .iter()
        .filter(|track| {
            track.id.clone().map_or(false, |track_id| {
                !original_track_ids.contains(&track_id)
            })
        })
        .map(|track| track.id.clone().unwrap())
        .collect::<Vec<String>>();

    let delete_tracks_bar = ProgressBar::new(spilled_track_ids.len() as u64);
    println!("Unsaving spilled tracks");

    for spilled_track_ids_chunk in spilled_track_ids.as_slice().chunks(50) {
        spotify.current_user_saved_tracks_delete(
            spilled_track_ids_chunk.to_vec(),
        )?;
        delete_tracks_bar.inc(spilled_track_ids_chunk.len() as u64);
    }

    delete_tracks_bar.finish();

    Ok(())
}

fn setup_auth(
    client_id: &str,
    client_secret: &str,
) -> Result<Spotify, Box<Error>> {
    let mut oauth = SpotifyOAuth::default()
        .scope(
            "user-library-read, user-library-modify, playlist-modify-private",
        )
        .client_id(client_id)
        .client_secret(client_secret)
        .redirect_uri("https://localhost:8888/srsa")
        .build();

    match get_token(&mut oauth) {
        Some(token_info) => {
            let client_credential = SpotifyClientCredentials::default()
                .token_info(token_info)
                .build();

            Ok(Spotify::default()
                .client_credentials_manager(client_credential)
                .build())
        }
        None => Err("Auth failed".into()),
    }
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "spotify-restore-saved-albums",
    about = "Restore saved albums for all saved tracks"
)]
struct CmdInput {
    #[structopt(short = "i", long = "client-id")]
    client_id: String,
    #[structopt(short = "s", long = "client-secret")]
    client_secret: String,
}

fn main() -> Result<(), Box<Error>> {
    let cmd_input = CmdInput::from_args();

    let spotify = setup_auth(&cmd_input.client_id, &cmd_input.client_secret)?;

    let saved_tracks = read_all_saved_tracks(&spotify)?;
    backup_saved_tracks(&spotify, saved_tracks.as_slice())?;
    let saved_albums = read_all_saved_albums(&spotify)?;
    delete_all_saved_albums(&spotify, saved_albums.as_slice())?;
    save_albums(&spotify, saved_tracks.as_slice())?;
    let spilled_saved_tracks = read_all_saved_tracks(&spotify)?;
    delete_spilled_saved_tracks(
        &spotify,
        saved_tracks.as_slice(),
        spilled_saved_tracks.as_slice(),
    )?;

    Ok(())
}
