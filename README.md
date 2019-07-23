# Spotify Restore Saved Albums

`spotify-restore-saved-albums` is a [CLI](https://en.wikipedia.org/wiki/Command-line_interface) tool that helps restoring saved albums from liked songs, similar to the  behavior pre June 2019 update.

WARNING use this at your own risk! This might removed ALL your saved albums. The author takes no responsibility for unexpected results.

### Behavior

- Lookup ALL saved tracks
- Crate playlist with ALL saved tracks called 'srsa backup'
- Remove ALL existing saved albums
- Go through ALL saved tracks and save each associated unique album in order of oldest saved tracks first, newest saved tracks last.

Note because the public save album API adds all tracks of the album to saved tracks, the tool has to repeat looking up all saved tracks, which now includes all original saved tracks plus all album tracks. To restore the original saved tracks it deletes all saved track that are not part of the original saved tracks.

Going forward if you want all your saved tracks to show up in the albums section, remember to always save both the tracks you are interested in AND the album.

Side effects:
- ALL saved albums without saved tracks are lost
- Added at time is updated to now for ALL saved tracks

## Getting Started

- Setup the prerequisites
- Close all spotify clients
- Run this tool like this and follow the instructions:
```
cargo run -- --release --client-id=75...80 --client-secret=13...c5
```

Note that client-id and client-secret are abbreviated in the example.

After running the tool, restart your spotify clients and you should now see ALL your albums again in the saved albums section.

### Prerequisites

A command line also known as console, beginner resources:
- https://www.webfx.com/blog/web-design/command-line/
- https://spideroak.support/hc/en-us/articles/115001893363
- https://en.wikipedia.org/wiki/Command-line_interface

Rust toolchain and cargo:
- See https://www.rust-lang.org/tools/install

Spotify developer app:
- Login into https://developer.spotify.com/dashboard
- Click 'CREATE A CLIENT ID'
- Fill out the fields and select non commercial
- Select your newly created app in the dashboard and click 'EDIT SETTINGS'
- Add `https://localhost:8888/srsa` to 'Redirect URIs'
- Click 'SAVE'
- Copy `Client ID` and `Client Secret`

### Installing

- Download this repository
- Run `cargo build` inside the repository directory, note this can take a couple minutes

## Running the tests

```
cargo test
```

## Contributing

Please read [CONTRIBUTING.md](CONTRIBUTING.md)
for details on our code of conduct, and the process for submitting pull requests to us.

## Versioning

We use [SemVer](http://semver.org/) for versioning. For the versions available,
see the [tags on this repository](https://github.com/Voultapher/spotify-restore-saved-albums/tags).

## Authors

* **Lukas Bergdoll** - *Initial work* - [Voultapher](https://github.com/Voultapher)

See also the list of [contributors](https://github.com/Voultapher/spotify-restore-saved-albums/contributors)
who participated in this project.

## License

This project is licensed under the Apache License, Version 2.0 -
see the [LICENSE](LICENSE) file for details.
