use serde::{Deserialize, Serialize};
use std::process::Command;
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::collections::HashMap;
use base64::{engine::general_purpose, Engine as _};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrackResult {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub duration: u64,
    pub thumbnail: String,
    pub source_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResolveResult {
    pub file_path: String,
    pub base64: String,
    pub mime_type: String,
    pub cached: bool,
}

static SEARCH_CACHE: OnceLock<Mutex<HashMap<String, (std::time::Instant, Vec<TrackResult>)>>> = OnceLock::new();

fn get_search_cache() -> &'static Mutex<HashMap<String, (std::time::Instant, Vec<TrackResult>)>> {
    SEARCH_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn get_cache_directory() -> PathBuf {
    let cache_dir = std::env::temp_dir().join("prosounds_cache");
    let _ = fs::create_dir_all(&cache_dir);
    cache_dir
}

pub fn search_tracks(query: &str, limit: u32) -> Result<Vec<TrackResult>, String> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return Ok(Vec::new());
    }

    // Check in-memory cache first (valid for 5 minutes)
    if let Ok(cache) = get_search_cache().lock() {
        if let Some((ts, results)) = cache.get(&q) {
            if ts.elapsed().as_secs() < 300 {
                return Ok(results.clone());
            }
        }
    }

    let search_term = format!("scsearch{}:{}", limit, query);

    let output = Command::new("yt-dlp")
        .args([
            "--flat-playlist",
            "--dump-json",
            "--no-warnings",
            "--skip-download",
            &search_term,
        ])
        .output()
        .map_err(|e| format!("Failed to run yt-dlp: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut tracks = Vec::new();

    for line in stdout.lines() {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
            let source_url = json["webpage_url"]
                .as_str()
                .or_else(|| json["url"].as_str())
                .unwrap_or("")
                .to_string();
            let id = json["id"]
                .as_str()
                .map(|s| s.to_string())
                .unwrap_or_else(|| source_url.clone());

            let mut best_thumb = json["thumbnail"].as_str().unwrap_or("").to_string();
            if let Some(thumbs) = json["thumbnails"].as_array() {
                for thumb in thumbs {
                    if let Some(tid) = thumb["id"].as_str() {
                        if tid == "t500x500" || tid == "crop" || tid == "large" {
                            if let Some(u) = thumb["url"].as_str() {
                                best_thumb = u.to_string();
                                break;
                            }
                        }
                    }
                }
            }

            tracks.push(TrackResult {
                id,
                title: json["title"].as_str().unwrap_or("Unknown").to_string(),
                artist: json["uploader"].as_str().unwrap_or("Unknown").to_string(),
                duration: json["duration"].as_u64().unwrap_or(0),
                thumbnail: best_thumb,
                source_url,
            });
        }
    }

    if let Ok(mut cache) = get_search_cache().lock() {
        cache.insert(q, (std::time::Instant::now(), tracks.clone()));
    }

    Ok(tracks)
}

pub fn find_cached_track_file(track_id: &str) -> Option<PathBuf> {
    let cache_dir = get_cache_directory();
    if let Ok(entries) = fs::read_dir(&cache_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                if stem == track_id {
                    if let Ok(meta) = fs::metadata(&path) {
                        if meta.len() > 1024 {
                            return Some(path);
                        }
                    }
                }
            }
        }
    }
    None
}

pub fn download_audio(source_url: &str, track_id: &str) -> Result<ResolveResult, String> {
    let cache_dir = get_cache_directory();

    if let Some(cached_path) = find_cached_track_file(track_id) {
        let bytes = fs::read(&cached_path).map_err(|e| format!("Failed to read cached file: {}", e))?;
        let b64 = general_purpose::STANDARD.encode(&bytes);
        let ext = cached_path.extension().and_then(|e| e.to_str()).unwrap_or("m4a");
        let mime_type = match ext {
            "mp3" => "audio/mpeg",
            "opus" => "audio/ogg",
            "wav" => "audio/wav",
            _ => "audio/mp4",
        }.to_string();
        return Ok(ResolveResult {
            file_path: cached_path.to_string_lossy().to_string(),
            base64: b64,
            mime_type,
            cached: true,
        });
    }

    let out_template = cache_dir.join(format!("{}.%(ext)s", track_id));

    let output = Command::new("yt-dlp")
        .args([
            "-N", "8",
            "-f", "bestaudio/best",
            "--no-warnings",
            "--no-playlist",
            "-o", &out_template.to_string_lossy(),
            source_url,
        ])
        .output()
        .map_err(|e| format!("Failed to run yt-dlp: {}", e))?;

    if !output.status.success() {
        let fallback_output = Command::new("yt-dlp")
            .args([
                "-N", "4",
                "--no-warnings",
                "--no-playlist",
                "-o", &out_template.to_string_lossy(),
                source_url,
            ])
            .output()
            .map_err(|e| format!("Fallback download failed: {}", e))?;

        if !fallback_output.status.success() {
            return Err(format!(
                "yt-dlp failed:\n{}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }

    let final_path = find_cached_track_file(track_id)
        .ok_or_else(|| "Audio file was not found after download.".to_string())?;

    let bytes = fs::read(&final_path).map_err(|e| format!("Failed to read downloaded file: {}", e))?;
    let b64 = general_purpose::STANDARD.encode(&bytes);

    let ext = final_path.extension().and_then(|e| e.to_str()).unwrap_or("m4a");
    let mime_type = match ext {
        "mp3" => "audio/mpeg",
        "opus" => "audio/ogg",
        "wav" => "audio/wav",
        _ => "audio/mp4",
    }.to_string();

    Ok(ResolveResult {
        file_path: final_path.to_string_lossy().to_string(),
        base64: b64,
        mime_type,
        cached: false,
    })
}

pub fn get_cache_total_bytes() -> u64 {
    let cache_dir = get_cache_directory();
    let mut total = 0;
    if let Ok(entries) = fs::read_dir(cache_dir) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                total += meta.len();
            }
        }
    }
    total
}

pub fn clear_audio_cache() -> Result<(), String> {
    let cache_dir = get_cache_directory();
    if let Ok(entries) = fs::read_dir(cache_dir) {
        for entry in entries.flatten() {
            let _ = fs::remove_file(entry.path());
        }
    }
    Ok(())
}
