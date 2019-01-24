use std::path::Path;
use std::process;

use crate::Result;

/// An aggregate of the information we need about a media file.
#[derive(Debug, Clone)]
pub struct Metadata {
    pub title: String,
    pub album: String,
    pub album_artist: String,
    pub track: usize,
    pub disc: usize,
}

impl Metadata {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Option<Self>> {
        get_raw_json(path).and_then(|x| {
            if let Some(x) = x {
                serde_json::from_slice::<serde_json::Value>(&x)
                    .map(Self::from_ffprobe_json)
                    .map_err(Into::into)
            } else {
                Ok(None)
            }
        })
    }

    fn from_ffprobe_json(v: serde_json::Value) -> Option<Self> {
        let tags = v
            .as_object()?
            .get("format")?
            .as_object()?
            .get("tags")?
            .as_object()?;

        Some(Self {
            title: tags.get("TITLE")?.as_str()?.to_string(),
            album: tags.get("ALBUM")?.as_str()?.to_string(),
            album_artist: tags.get("album_artist")?.as_str()?.to_string(),
            track: tags.get("track")?.as_str()?.parse().unwrap(),
            disc: tags.get("disc")?.as_str()?.parse().unwrap(),
        })
    }
}

fn get_raw_json<P: AsRef<Path>>(path: P) -> Result<Option<Vec<u8>>> {
    let mut cmd = process::Command::new("ffprobe");

    // Prevent FFprobe from displaying any extra stuff.
    cmd.arg("-v").arg("quiet");

    // Request JSON formatted output.
    cmd.arg("-print_format").arg("json");

    // Show container information.
    cmd.arg("-show_format");

    cmd.arg(path.as_ref());

    let output = cmd.output()?;

    Ok(if output.status.success() {
        Some(output.stdout)
    } else {
        None
    })
}
