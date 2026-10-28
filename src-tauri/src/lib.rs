mod soundcloud;

use soundcloud::{
    clear_audio_cache, download_audio, get_cache_total_bytes, search_tracks, ResolveResult, TrackResult,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Playlist {
    pub id: String,
    pub name: String,
    pub tracks: Vec<TrackResult>,
}

fn playlists_path(app: &tauri::AppHandle) -> PathBuf {
    let mut path = app.path().app_data_dir().expect("no app data dir");
    fs::create_dir_all(&path).ok();
    path.push("playlists.json");
    path
}

fn load_playlists(app: &tauri::AppHandle) -> Vec<Playlist> {
    let path = playlists_path(app);
    if !path.exists() {
        return Vec::new();
    }
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_playlists(app: &tauri::AppHandle, playlists: &[Playlist]) -> Result<(), String> {
    let path = playlists_path(app);
    let json = serde_json::to_string_pretty(playlists).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

#[tauri::command]
fn search(query: String) -> Result<Vec<TrackResult>, String> {
    search_tracks(&query, 20)
}

#[tauri::command]
fn resolve(url: String, id: String) -> Result<ResolveResult, String> {
    download_audio(&url, &id)
}

#[tauri::command]
fn preload_track(url: String, id: String) -> Result<(), String> {
    std::thread::spawn(move || {
        let _ = download_audio(&url, &id);
    });
    Ok(())
}

#[tauri::command]
fn get_playlists(app: tauri::AppHandle) -> Vec<Playlist> {
    load_playlists(&app)
}

#[tauri::command]
fn create_playlist(app: tauri::AppHandle, name: String) -> Result<Vec<Playlist>, String> {
    let mut playlists = load_playlists(&app);
    let id = format!(
        "pl_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    );
    playlists.push(Playlist {
        id,
        name,
        tracks: Vec::new(),
    });
    save_playlists(&app, &playlists)?;
    Ok(playlists)
}

#[tauri::command]
fn delete_playlist(app: tauri::AppHandle, id: String) -> Result<Vec<Playlist>, String> {
    let mut playlists = load_playlists(&app);
    playlists.retain(|p| p.id != id);
    save_playlists(&app, &playlists)?;
    Ok(playlists)
}

#[tauri::command]
fn add_to_playlist(app: tauri::AppHandle, playlist_id: String, track: TrackResult) -> Result<Vec<Playlist>, String> {
    let mut playlists = load_playlists(&app);
    if let Some(pl) = playlists.iter_mut().find(|p| p.id == playlist_id) {
        if !pl.tracks.iter().any(|t| t.id == track.id) {
            pl.tracks.push(track);
        }
    }
    save_playlists(&app, &playlists)?;
    Ok(playlists)
}

#[tauri::command]
fn remove_from_playlist(app: tauri::AppHandle, playlist_id: String, track_id: String) -> Result<Vec<Playlist>, String> {
    let mut playlists = load_playlists(&app);
    if let Some(pl) = playlists.iter_mut().find(|p| p.id == playlist_id) {
        pl.tracks.retain(|t| t.id != track_id);
    }
    save_playlists(&app, &playlists)?;
    Ok(playlists)
}

#[tauri::command]
fn get_cache_stats() -> u64 {
    get_cache_total_bytes()
}

#[tauri::command]
fn clear_cache() -> Result<(), String> {
    clear_audio_cache()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            search,
            resolve,
            preload_track,
            get_playlists,
            create_playlist,
            delete_playlist,
            add_to_playlist,
            remove_from_playlist,
            get_cache_stats,
            clear_cache
        ])
        .run(tauri::generate_context!())
        .expect("error while running Prosounds");
}
