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

    if !dst.exists() {
        fs::create_dir_all(&dst)?;
    }

    for entry in it {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }

        if let Some(song) = metadata::Metadata::new(entry.path())? {
            let artist_path = dst.join(&song.album_artist);
            if !artist_path.exists() {
                fs::create_dir(&artist_path)?;
            }

            let album_path = artist_path.join(&song.album);
            if !album_path.exists() {
                fs::create_dir(&album_path)?;
            }

            let dst_path = album_path.join(file_name(&song));

            // If there exists a file there already, we perform no work.
            if !dst_path.exists() {
                convert(entry.path(), dst_path, song.container_format)?;
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
    let mut s = format!(
        "Disc {} - {:02} - {}{}",
        disc,
        track,
        without_slashes(title.clone()),
        suffix
    );
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

/// Returns a string with slashes replaced with hyphens.
///
/// Useful because some song titles, stored as metadata tags in media files, contain slashes and
/// thus cannot be used as file names.
fn without_slashes(s: String) -> String {
    let mut v = s.into_bytes();

    for b in v.iter_mut() {
        if *b == b'/' {
            *b = b'-';
        }
    }

    String::from_utf8(v).unwrap()
}
