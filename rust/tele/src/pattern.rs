use std::{path::PathBuf, time::Instant, fs};
use anyhow::anyhow;
use tracing::{error, debug};

use funscript::FScript;

pub fn get_pattern_names(pattern_path: &str, vibration_patterns: bool) -> Vec<String> {
    match get_pattern_paths(pattern_path) {
        Ok(patterns) => patterns
            .iter()
            .filter(|p| p.is_vibration == vibration_patterns)
            .map(|p| p.name.clone())
            .collect::<Vec<String>>(),
        Err(err) => {
            error!("Failed reading patterns {}", err);
            vec![]
        }
    }
}

struct TkPatternFile {
    path: PathBuf,
    is_vibration: bool,
    name: String,
}

fn get_pattern_paths(pattern_path: &str) -> Result<Vec<TkPatternFile>, anyhow::Error> {
    let mut patterns = vec![];
    let pattern_dir = fs::read_dir(pattern_path)?;
    for entry in pattern_dir {
        let file = entry?;

        let path = file.path();
        let path_clone = path.clone();
        let file_name = path
            .file_name()
            .ok_or_else(|| anyhow!("No file name"))?
            .to_str()
            .ok_or_else(|| anyhow!("Invalid unicode"))?;
        if !file_name.to_lowercase().ends_with(".funscript") {
            continue;
        }

        let is_vibration = file_name.to_lowercase().ends_with(".vibrator.funscript");
        let removal: usize = if is_vibration {
            file_name.len() - ".vibrator.funscript".len()
        } else {
            file_name.len() - ".funscript".len()
        };

        patterns.push(TkPatternFile {
            path: path_clone,
            is_vibration,
            name: String::from(&file_name[0..removal]),
        })
    }
    Ok(patterns)
}

pub fn read_pattern(
    pattern_path: &str,
    pattern_name: &str,
    vibration_pattern: bool,
) -> Option<FScript> {
    match read_pattern_name(pattern_path, pattern_name, vibration_pattern) {
        Ok(funscript) => Some(funscript),
        Err(err) => {
            error!(
                "Error loading funscript vibration pattern={} err={}",
                pattern_name, err
            );
            None
        }
    }
}

pub fn read_pattern_name(
    pattern_path: &str,
    pattern_name: &str,
    vibration_pattern: bool,
) -> Result<FScript, anyhow::Error> {
    let now = Instant::now();
    let patterns: Vec<TkPatternFile> = get_pattern_paths(pattern_path)?;
    let pattern = patterns
        .iter()
        .find(|d| {
            d.is_vibration == vibration_pattern
                && d.name.to_lowercase() == pattern_name.to_lowercase()
        })
        .ok_or_else(|| anyhow!("Pattern '{}' not found", pattern_name))?;

    let fs = funscript::load_funscript(pattern.path.to_str().unwrap())?;
    debug!("Read pattern {} in {:?}", pattern_name, now.elapsed());
    Ok(fs)
}
