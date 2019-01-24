use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process;

use structopt::StructOpt;

use cli::Cli;

mod cli;
mod metadata;

type Error = Box<std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let Cli { src, dst } = Cli::from_args();

    let it = walkdir::WalkDir::new(src);
    let mut map = HashMap::new();

    for entry in it {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }

        if let Some(m) = metadata::Metadata::new(entry.path())? {
            map.entry(m.album_artist.clone())
                .or_insert(HashMap::new())
                .entry(m.album.clone())
                .or_insert(Vec::new())
                .push((entry.into_path(), m));
        }
    }

    for (artist, albums) in map {
        let artist_path = dst.join(artist);
        if !artist_path.exists() {
            fs::create_dir(&artist_path)?;
        }
        for (album, songs) in albums {
            let album_path = artist_path.join(album);
            if !album_path.exists() {
                fs::create_dir(&album_path)?;
            }

            for (src_path, song) in songs {
                let dst_path = album_path.join(file_name(&song));
                convert(src_path, dst_path, song.container_format)?;
            }
        }
    }

    Ok(())
}

fn convert<P, Q>(src: P, dst: Q, container_format: metadata::ContainerFormat) -> Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let src = src.as_ref().as_os_str();
    let dst = dst.as_ref().as_os_str();

    let mut cmd = process::Command::new("ffmpeg");

    cmd.arg("-hide_banner").arg("-i").arg(src);

    if let metadata::ContainerFormat::Ogg = container_format {
        cmd.arg("-map_metadata");
        cmd.arg("0:s:0");
    }

    cmd.arg(dst);

    let status = cmd.status()?;

    if !status.success() {
        Err("ffmpeg invocation failed".into())
    } else {
        Ok(())
    }
}

const FILE_NAME_MAXIMUM_LENGTH: usize = 63;

fn file_name(
    metadata::Metadata {
        title, disc, track, ..
    }: &metadata::Metadata,
) -> String {
    let suffix = ".mp3";
    let mut s = format!("Disc {} - {:02} - {}{}", disc, track, title, suffix);
    if FILE_NAME_MAXIMUM_LENGTH < s.len() {
        let more_indicator = "---";
        while FILE_NAME_MAXIMUM_LENGTH < s.len() + more_indicator.len() + suffix.len() {
            s.pop();
        }
        s.push_str(more_indicator);
        s.push_str(suffix);
    }
    s
}
