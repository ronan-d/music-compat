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
            let metadata::Metadata { album, track, disc } = m;
            map.entry(album)
                .or_insert(Vec::new())
                .push((entry.into_path(), disc, track));
        }
    }

    for (album, songs) in map {
        let album_path = dst.join(album);
        if !album_path.exists() {
            fs::create_dir(&album_path)?;
        }

        for (src_path, disc, track) in songs {
            let dst_path = album_path.join(format!("{:04}-{:04}.mp3", disc, track));
            convert(src_path, dst_path)?;
        }
    }

    Ok(())
}

fn convert<P, Q>(src: P, dst: Q) -> Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let src = src.as_ref().as_os_str();
    let dst = dst.as_ref().as_os_str();

    let status = process::Command::new("ffmpeg")
        .arg("-hide_banner")
        .arg("-i")
        .arg(src)
        .arg(dst)
        .status()?;

    if !status.success() {
        Err("ffmpeg invocation failed".into())
    } else {
        Ok(())
    }
}
