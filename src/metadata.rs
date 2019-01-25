use std::path::Path;
use std::process;

use crate::Result;

/// An aggregate of the information we need about a media file.
#[derive(Debug, Clone)]
pub struct Metadata {
    pub container_format: ContainerFormat,
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
        let format = v.as_object()?.get("format")?.as_object()?;

        let format_name = format.get("format_name")?.as_str()?;

        let container_format = ContainerFormat::from_name(format_name)?;

        // This come from an analysis of FFprobe's output.
        let tags = match container_format {
            ContainerFormat::Flac => format.get("tags")?.as_object()?,
            ContainerFormat::Ogg => v
                .as_object()?
                .get("streams")?
                .as_array()?
                .first()?
                .as_object()?
                .get("tags")?
                .as_object()?,
        };

        Some(Self {
            container_format,
            title: tags.get("TITLE")?.as_str()?.to_string(),
            album: tags.get("ALBUM")?.as_str()?.to_string(),
            album_artist: tags.get("album_artist")?.as_str()?.to_string(),
            track: trim_index(tags.get("track")?.as_str()?)?.parse().unwrap(),
            disc: tags
                .get("disc")
                .and_then(serde_json::Value::as_str)
                .and_then(trim_index)
                .and_then(|x| x.parse().ok())
                .unwrap_or(1),
        })
    }
}

fn get_raw_json<P: AsRef<Path>>(path: P) -> Result<Option<Vec<u8>>> {
    let mut cmd = process::Command::new("ffprobe");

    // Prevent FFprobe from displaying any extra stuff.
    cmd.arg("-v").arg("quiet");

    // Request JSON formatted output.
    cmd.arg("-print_format").arg("json");

    // Show container format information.
    cmd.arg("-show_format");

    // Show stream information.
    cmd.arg("-show_streams");

    cmd.arg(path.as_ref());

    let output = cmd.output()?;

    Ok(if output.status.success() {
        Some(output.stdout)
    } else {
        None
    })
}

#[derive(Debug, Clone, Copy)]
pub enum ContainerFormat {
    Ogg,
    Flac,
}

impl ContainerFormat {
    fn from_name(name: &str) -> Option<Self> {
        use self::ContainerFormat::*;
        match name {
            "ogg" => Some(Ogg),
            "flac" => Some(Flac),
            _ => None,
        }
    }
}

/// Filters a track number.
///
/// Useful because some `track` metadata tags include the total number of tracks, like in `7/20`
/// insteand of just `7`. The same thing happens for disc numbers.
fn trim_index(s: &str) -> Option<&str> {
    s.split(|c: char| !c.is_digit(10)).next()
}
