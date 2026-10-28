<script>
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, onDestroy } from "svelte";
  import { fade, fly, scale, slide } from "svelte/transition";

  // --- User Profile & Onboarding ---
  let userName = "";
  let tempNameInput = "";
  let showOnboarding = false;
  let showIntroLoading = true;
  let introProgress = 0;

  // --- Core Navigation & Search ---
  let query = "";
  let searchInputEl;
  let results = [];
  let playlists = [];
  let activePlaylistId = null;
  let currentTrack = null;
  let loading = false;
  let error = "";
  let toastMessage = "";
  let toastTimeout;

  let showNewPlaylist = false;
  let newPlaylistName = "";
  let addMenuTrackId = null;
  let resolvingId = null;
  let activeTab = "recommended"; // "recommended" | "search" | "queue" | "playlists" | "eq" | "settings"

  // --- Audio State ---
  let audio = null;
  let isPlaying = false;
  let currentTime = 0;
  let duration = 0;
  let volume = 0.85;
  let isMuted = false;
  let previousVolume = 0.85;
  let repeatMode = "off"; // "off" | "all" | "one"
  let shuffle = false;
  let queue = [];
  let queueIndex = 0;
  let isSeeking = false;
  let hoverTime = null;
  let hoverPct = 0;
  let activeBlobUrl = null;
  let cacheSizeBytes = 0;

  // --- Auto Audio Normalizer & Smooth Leveling ---
  let autoNormalizer = true;
  let targetVolume = 0.85;
  let fadeInterval = null;

  // --- Visualizer & 120 FPS Ambient Particles ---
  let visualizerCanvas = null;
  let particlesCanvas = null;
  let visualizerAnimId = null;
  let particlesAnimId = null;
  let lastFrameTime = 0;
  let audioBassPulse = 0;

  // --- Appearance & 5+ Themes + Custom Theme Creator ---
  let theme = "midnight"; // "midnight" | "cyberpunk" | "sakura" | "sunset" | "emerald" | "custom"
  let customAccent = "#a6c1ee";
  let customBgDarkness = 0.65;
  let customGlow = true;
  let bgDarkness = 0.65;
  let bgBlur = 0;
  let animatedParticles = true;

  // --- Mascot Expressions & Click Rotation ---
  let showMascot = true;
  let mascotClickIndex = 0;
  let currentMascotMood = "happy";
  let mascotMessage = "Welcome to ProSounds! Click on me anytime nya~";
  let showBubble = true;
  let bubbleTimeout;
  let mascotSpinning = false;

  // Clean 6 cropped mascot cutouts
  const mascotExpressions = [
    { id: "happy", file: "/mascot_cutouts/mascot_happy.png", label: "Happy" },
    { id: "vibe", file: "/mascot_cutouts/mascot_vibe.png", label: "Vibing" },
    { id: "cute", file: "/mascot_cutouts/mascot_cute.png", label: "Cute" },
    { id: "nya", file: "/mascot_cutouts/mascot_nya.png", label: "Nya~" },
    { id: "wink", file: "/mascot_cutouts/mascot_wink.png", label: "Wink" },
    { id: "sleep", file: "/mascot_cutouts/mascot_sleep.png", label: "Resting" }
  ];

  const cuteVoicelines = [
    "Nyaaa~! What track are we vibing to, Senpai?~",
    "Uwah, this sound is so clean nya!",
    "Ganbatte! You have top-tier music taste nya~",
    "Ara ara~ Relax and let the rhythm flow nya!",
    "Nya~ ProSounds is running at ultra-smooth 120 FPS!",
    "Sugoi! That bass hit just right nya~",
    "Puffed up with pure cozy vibes nya! :3",
    "Keep vibing, Senpai! I'm cheering for you nya~"
  ];

  // Curated Recommended Tracks
  const curatedPicks = [
    { title: "Lofi Hip Hop Radio - Beats to Relax/Study to", artist: "Lofi Girl", genre: "Lo-Fi", query: "lofi hip hop radio beats to relax study to" },
    { title: "Resonance (Synthwave Nostalgia)", artist: "HOME", genre: "Synthwave", query: "home resonance" },
    { title: "Midnight City Vibe", artist: "M83", genre: "Synthwave", query: "midnight city" },
    { title: "Ketsumeishi - Sakura (Acoustic Chill)", artist: "Anime Chill", genre: "Anime", query: "sakura acoustic chill lofi" },
    { title: "Tokyo Drift Phonk Edition", artist: "DVRST", genre: "Phonk", query: "dvrst close eyes phonk" },
    { title: "Weightless - Ambient Deep Sleep", artist: "Marconi Union", genre: "Ambient", query: "marconi union weightless" },
    { title: "Sunflower - Chill Acoustic Mix", artist: "Post Malone", genre: "Chillhop", query: "sunflower acoustic chill" },
    { title: "Snowfall (Slowed + Reverb)", artist: "\xD8neheart x reidenshi", genre: "Ambient", query: "oneheart snowfall slowed" }
  ];

  // --- Cute Anime Chime via Web Audio Synthesis ---
  function playCuteAnimeChime() {
    try {
      const AudioCtx = window.AudioContext || window.webkitAudioContext;
      if (!AudioCtx) return;
      const ctx = new AudioCtx();
      const now = ctx.currentTime;

      const osc = ctx.createOscillator();
      const gain = ctx.createGain();

      osc.type = "sine";
      // Sweet two-tone ascending cute chime (880Hz -> 1320Hz)
      osc.frequency.setValueAtTime(880, now);
      osc.frequency.exponentialRampToValueAtTime(1320, now + 0.08);
      osc.frequency.exponentialRampToValueAtTime(1100, now + 0.18);

      gain.gain.setValueAtTime(0.08, now);
      gain.gain.exponentialRampToValueAtTime(0.001, now + 0.25);

      osc.connect(gain);
      gain.connect(ctx.destination);

      osc.start(now);
      osc.stop(now + 0.25);
    } catch (e) {
      // Audio fallback
    }
  }

  function handleMascotClick() {
    playCuteAnimeChime();
    mascotSpinning = true;
    setTimeout(() => (mascotSpinning = false), 500);

    // Rotate to next facial expression every single click
    mascotClickIndex = (mascotClickIndex + 1) % mascotExpressions.length;
    currentMascotMood = mascotExpressions[mascotClickIndex].id;

    // Pick dynamic anime voiceline
    const line = cuteVoicelines[Math.floor(Math.random() * cuteVoicelines.length)];
    triggerMascot(line);
  }

  function showToast(msg) {
    toastMessage = msg;
    clearTimeout(toastTimeout);
    toastTimeout = setTimeout(() => {
      toastMessage = "";
    }, 2400);
  }

  function triggerMascot(msg) {
    mascotMessage = msg;
    showBubble = true;
    clearTimeout(bubbleTimeout);
    bubbleTimeout = setTimeout(() => {
      showBubble = false;
    }, 4200);
  }

  // --- Smooth Audio Leveling & Crossfade ---
  function smoothVolumeTransition(targetVol, durationMs = 250) {
    if (!audio) return;
    clearInterval(fadeInterval);
    const startVol = audio.volume;
    const startTime = performance.now();

    fadeInterval = setInterval(() => {
      const elapsed = performance.now() - startTime;
      const progress = Math.min(1, elapsed / durationMs);
      audio.volume = startVol + (targetVol - startVol) * progress;
      if (progress >= 1) {
        clearInterval(fadeInterval);
      }
    }, 16);
  }

  // --- 120 FPS Native Canvas Visualizer & Ambient Particles ---
  function startCanvases() {
    let particles = [];
    const pCount = 38;

    for (let i = 0; i < pCount; i++) {
      particles.push({
        x: Math.random() * window.innerWidth,
        y: Math.random() * window.innerHeight,
        radius: Math.random() * 2.2 + 0.8,
        speedX: (Math.random() - 0.5) * 0.4,
        speedY: -Math.random() * 0.5 - 0.2,
        opacity: Math.random() * 0.6 + 0.2
      });
    }

    const renderLoop = (time) => {
      visualizerAnimId = requestAnimationFrame(renderLoop);

      // 1. Render Ambient Floating Particles
      if (particlesCanvas && animatedParticles) {
        const pCtx = particlesCanvas.getContext("2d");
        const pw = particlesCanvas.width;
        const ph = particlesCanvas.height;
        pCtx.clearRect(0, 0, pw, ph);

        for (let p of particles) {
          p.x += p.speedX;
          p.y += p.speedY;

          if (p.y < 0) { p.y = ph; p.x = Math.random() * pw; }
          if (p.x < 0) p.x = pw;
          if (p.x > pw) p.x = 0;

          pCtx.beginPath();
          pCtx.arc(p.x, p.y, p.radius * (1 + audioBassPulse * 1.5), 0, Math.PI * 2);
          pCtx.fillStyle = `rgba(255, 255, 255, ${p.opacity * (0.4 + audioBassPulse * 0.5)})`;
          pCtx.fill();
        }
      }

      // 2. Render Audio Spectrum Visualizer
      if (visualizerCanvas) {
        const vCtx = visualizerCanvas.getContext("2d");
        const vw = visualizerCanvas.width;
        const vh = visualizerCanvas.height;
        vCtx.clearRect(0, 0, vw, vh);

        if (!isPlaying) {
          vCtx.fillStyle = "rgba(255, 255, 255, 0.15)";
          vCtx.fillRect(0, vh - 2, vw, 2);
          audioBassPulse = 0;
          return;
        }

        const t = time * 0.0035;
        const barCount = 26;
        const barW = (vw / barCount) - 2;
        let bassSum = 0;

        for (let i = 0; i < barCount; i++) {
          const factor = Math.sin(t * 3.2 + i * 0.45) * 0.5 + 0.5;
          const wave = Math.cos(t * 2.1 + i * 0.3) * 0.3 + 0.7;
          const barH = Math.max(3, factor * wave * (vh * 0.9));

          if (i < 5) bassSum += factor;

          const x = i * (barW + 2);
          const y = vh - barH;

          const grad = vCtx.createLinearGradient(0, vh, 0, y);
          const activeAccent = theme === "custom" ? customAccent : getThemeColor(theme);
          grad.addColorStop(0, "rgba(255, 255, 255, 0.15)");
          grad.addColorStop(1, activeAccent);

          vCtx.fillStyle = grad;
          vCtx.beginPath();
          vCtx.roundRect(x, y, barW, barH, [2, 2, 0, 0]);
          vCtx.fill();
        }

        audioBassPulse = (bassSum / 5) * 0.035;
      }
    };

    visualizerAnimId = requestAnimationFrame(renderLoop);
  }

  function getThemeColor(t) {
    if (t === "cyberpunk") return "#00f0ff";
    if (t === "sakura") return "#ff7597";
    if (t === "sunset") return "#ff8c42";
    if (t === "emerald") return "#10b981";
    return "#a6c1ee";
  }

  onMount(async () => {
    // 1. Check Onboarding
    const savedName = localStorage.getItem("prosounds_user");
    if (!savedName) {
      showOnboarding = true;
    } else {
      userName = savedName;
    }

    // 2. Load Theme & Settings
    const savedTheme = localStorage.getItem("prosounds_theme");
    if (savedTheme) theme = savedTheme;
    const savedAccent = localStorage.getItem("prosounds_custom_accent");
    if (savedAccent) customAccent = savedAccent;

    // 3. Intro Loading Screen sequence
    let progressTimer = setInterval(() => {
      introProgress += 20;
      if (introProgress >= 100) {
        clearInterval(progressTimer);
        setTimeout(() => (showIntroLoading = false), 350);
      }
    }, 70);

    // 4. Setup Audio Player
    audio = new Audio();
    audio.volume = volume;

    audio.addEventListener("timeupdate", () => {
      if (!isSeeking) currentTime = audio.currentTime;
    });
    audio.addEventListener("loadedmetadata", () => {
      duration = audio.duration || 0;
    });
    audio.addEventListener("ended", onTrackEnd);
    audio.addEventListener("play", () => {
      isPlaying = true;
      if (autoNormalizer) smoothVolumeTransition(targetVolume);
    });
    audio.addEventListener("pause", () => {
      isPlaying = false;
    });

    try {
      playlists = await invoke("get_playlists");
      updateCacheStats();
    } catch (e) {
      console.warn(e);
    }

    window.addEventListener("keydown", handleKeydown);
    window.addEventListener("resize", handleResize);
    handleResize();
    startCanvases();
  });

  onDestroy(() => {
    if (audio) {
      audio.pause();
      audio.src = "";
    }
    if (activeBlobUrl) URL.revokeObjectURL(activeBlobUrl);
    if (visualizerAnimId) cancelAnimationFrame(visualizerAnimId);
    window.removeEventListener("keydown", handleKeydown);
    window.removeEventListener("resize", handleResize);
  });

  function handleResize() {
    if (particlesCanvas) {
      particlesCanvas.width = window.innerWidth;
      particlesCanvas.height = window.innerHeight;
    }
  }

  function saveUserName() {
    if (!tempNameInput.trim()) return;
    userName = tempNameInput.trim();
    localStorage.setItem("prosounds_user", userName);
    showOnboarding = false;
    triggerMascot(`Konnichiwa, ${userName} nya~! Let's find your favorite tunes!`);
  }

  function applyTheme(newTheme) {
    theme = newTheme;
    localStorage.setItem("prosounds_theme", newTheme);
    showToast(`Theme: ${newTheme.toUpperCase()}`);
  }

  function saveCustomTheme() {
    localStorage.setItem("prosounds_theme", "custom");
    localStorage.setItem("prosounds_custom_accent", customAccent);
    theme = "custom";
    showToast("Custom theme saved!");
  }

  async function updateCacheStats() {
    try {
      cacheSizeBytes = await invoke("get_cache_stats");
    } catch (e) {
      console.warn(e);
    }
  }

  function handleKeydown(e) {
    if (["INPUT", "TEXTAREA", "SELECT"].includes(document.activeElement?.tagName)) {
      if (e.key === "Escape") document.activeElement.blur();
      return;
    }

    switch (e.key) {
      case " ":
        e.preventDefault();
        togglePlay();
        break;
      case "ArrowRight":
        e.preventDefault();
        seekRelative(5);
        break;
      case "ArrowLeft":
        e.preventDefault();
        seekRelative(-5);
        break;
      case "ArrowUp":
        e.preventDefault();
        changeVolume(0.05);
        break;
      case "ArrowDown":
        e.preventDefault();
        changeVolume(-0.05);
        break;
      case "m":
      case "M":
        toggleMute();
        break;
      case "n":
      case "N":
        playNextTrack();
        break;
      case "p":
      case "P":
        playPrevTrack();
        break;
      case "s":
      case "S":
        shuffle = !shuffle;
        showToast(shuffle ? "Shuffle On" : "Shuffle Off");
        break;
      case "l":
      case "L":
        cycleRepeat();
        break;
      case "/":
        e.preventDefault();
        if (searchInputEl) searchInputEl.focus();
        break;
    }
  }

  function seekRelative(seconds) {
    if (!audio || !duration) return;
    audio.currentTime = Math.max(0, Math.min(duration, audio.currentTime + seconds));
  }

  function changeVolume(delta) {
    volume = Math.max(0, Math.min(1, volume + delta));
    targetVolume = volume;
    if (audio) audio.volume = volume;
    if (isMuted && volume > 0) isMuted = false;
  }

  function toggleMute() {
    if (!audio) return;
    if (isMuted) {
      volume = previousVolume || 0.5;
      audio.volume = volume;
      isMuted = false;
      showToast("Unmuted");
    } else {
      previousVolume = volume;
      volume = 0;
      audio.volume = 0;
      isMuted = true;
      showToast("Muted");
    }
  }

  async function doSearch() {
    if (!query.trim()) return;
    loading = true;
    error = "";
    activeTab = "search";
    triggerMascot("Searching SoundCloud nya~");

    try {
      results = await invoke("search", { query: query.trim() });
      if (results.length > 0) {
        triggerMascot(`Found ${results.length} bangers nya!`);
      } else {
        triggerMascot("No results found. Try another query!");
      }
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  function quickSearch(keyword) {
    query = keyword;
    doSearch();
  }

  async function playCurated(pick) {
    query = pick.query;
    activeTab = "search";
    loading = true;
    triggerMascot(`Loading ${pick.title.slice(0, 20)} nya~`);
    try {
      results = await invoke("search", { query: pick.query });
      if (results.length > 0) {
        playTrack(results[0], results);
      }
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function playTrack(track, list = null) {
    error = "";
    resolvingId = track.id;
    triggerMascot(`Preparing ${track.title.slice(0, 22)} nya~`);

    try {
      const res = await invoke("resolve", { url: track.source_url, id: track.id });

      // Fast binary conversion to in-memory Blob URL
      const binaryString = atob(res.base64);
      const len = binaryString.length;
      const bytes = new Uint8Array(len);
      for (let i = 0; i < len; i++) {
        bytes[i] = binaryString.charCodeAt(i);
      }
      const blob = new Blob([bytes], { type: res.mime_type || "audio/mp4" });

      if (activeBlobUrl) URL.revokeObjectURL(activeBlobUrl);
      activeBlobUrl = URL.createObjectURL(blob);

      if (autoNormalizer) {
        audio.volume = 0.2; // soft start
      }
      audio.src = activeBlobUrl;
      await audio.play();

      currentTrack = track;
      if (res.cached) {
        showToast("Instant cache hit (0ms)");
      }

      // Auto update mascot mood based on track
      const songMood = detectSongMood(track.title, track.artist);
      currentMascotMood = songMood;
      triggerMascot(`Vibing to: ${track.title.slice(0, 26)} nya~!`);

      if (list) {
        queue = list;
        queueIndex = list.findIndex((t) => t.id === track.id);
      } else if (!queue.some(t => t.id === track.id)) {
        queue = [...queue, track];
        queueIndex = queue.length - 1;
      }

      // Background preload next track
      preloadNextInQueue();
      updateCacheStats();
    } catch (e) {
      console.error(e);
      error = String(e);
      triggerMascot("Could not play track nya. Trying another!");
    } finally {
      resolvingId = null;
    }
  }

  function detectSongMood(title = "", artist = "") {
    const text = `${title} ${artist}`.toLowerCase();
    if (text.match(/chill|lo-?fi|relax|sleep|calm|rain|coffee|study|night|warm|ambient/)) return "vibe";
    if (text.match(/phonk|bass|drill|trap|rave|hype|party|club|rock|dance/)) return "happy";
    if (text.match(/sing|vocal|acoustic|piano|sweet|love|heart|pop|anime/)) return "nya";
    return "cute";
  }

  function preloadNextInQueue() {
    if (queue.length > 1 && queueIndex < queue.length - 1) {
      const nextTrack = queue[queueIndex + 1];
      invoke("preload_track", { url: nextTrack.source_url, id: nextTrack.id }).catch(() => {});
    }
  }

  function onTrackEnd() {
    if (repeatMode === "one") {
      audio.currentTime = 0;
      audio.play();
      return;
    }
    if (shuffle && queue.length > 1) {
      let next = queueIndex;
      while (next === queueIndex) next = Math.floor(Math.random() * queue.length);
      queueIndex = next;
      playTrack(queue[next], queue);
      return;
    }
    if (queueIndex < queue.length - 1) {
      queueIndex++;
      playTrack(queue[queueIndex], queue);
      return;
    }
    if (repeatMode === "all" && queue.length > 0) {
      queueIndex = 0;
      playTrack(queue[0], queue);
    }
  }

  function playNextTrack() {
    if (queue.length === 0) return;
    if (shuffle && queue.length > 1) {
      let next = queueIndex;
      while (next === queueIndex) next = Math.floor(Math.random() * queue.length);
      queueIndex = next;
      playTrack(queue[next], queue);
    } else if (queueIndex < queue.length - 1) {
      queueIndex++;
      playTrack(queue[queueIndex], queue);
    } else {
      queueIndex = 0;
      playTrack(queue[0], queue);
    }
  }

  function playPrevTrack() {
    if (!audio) return;
    if (audio.currentTime > 3) {
      audio.currentTime = 0;
      return;
    }
    if (queueIndex > 0) {
      queueIndex--;
      playTrack(queue[queueIndex], queue);
    } else if (queue.length > 0) {
      queueIndex = queue.length - 1;
      playTrack(queue[queueIndex], queue);
    }
  }

  function togglePlay() {
    if (!audio || !currentTrack) return;
    if (isPlaying) {
      audio.pause();
      currentMascotMood = "sleep";
      triggerMascot("Paused. Taking a quick catnap nya~");
    } else {
      audio.play();
      currentMascotMood = "vibe";
      triggerMascot("Back to the rhythm nya!");
    }
  }

  function handleSeekMouseDown(e) {
    isSeeking = true;
    updateSeekFromEvent(e);
  }

  function handleSeekMouseMove(e) {
    const rect = e.currentTarget.getBoundingClientRect();
    const pct = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
    hoverPct = pct;
    hoverTime = pct * duration;
    if (isSeeking) {
      currentTime = pct * duration;
    }
  }

  function handleSeekMouseUp(e) {
    if (isSeeking) {
      isSeeking = false;
      updateSeekFromEvent(e);
    }
  }

  function updateSeekFromEvent(e) {
    if (!audio || !duration) return;
    const rect = e.currentTarget.getBoundingClientRect();
    const pct = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
    audio.currentTime = pct * duration;
    currentTime = audio.currentTime;
  }

  function setVolumeFromInput(e) {
    volume = parseFloat(e.target.value);
    targetVolume = volume;
    if (audio) audio.volume = volume;
    if (volume === 0) isMuted = true;
    else isMuted = false;
  }

  function cycleRepeat() {
    if (repeatMode === "off") {
      repeatMode = "all";
      showToast("Repeat All");
    } else if (repeatMode === "all") {
      repeatMode = "one";
      showToast("Repeat One");
    } else {
      repeatMode = "off";
      showToast("Repeat Off");
    }
  }

  function fmt(sec) {
    if (!sec || isNaN(sec)) return "0:00";
    const m = Math.floor(sec / 60);
    const s = String(Math.floor(sec % 60)).padStart(2, "0");
    return `${m}:${s}`;
  }

  function formatBytes(bytes) {
    if (!bytes) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
  }

  async function createPlaylist() {
    if (!newPlaylistName.trim()) return;
    try {
      playlists = await invoke("create_playlist", { name: newPlaylistName.trim() });
      showToast(`Playlist "${newPlaylistName.trim()}" created`);
      newPlaylistName = "";
      showNewPlaylist = false;
    } catch (e) {
      error = String(e);
    }
  }

  async function deletePlaylist(id, name) {
    if (!confirm(`Delete playlist "${name}"?`)) return;
    try {
      playlists = await invoke("delete_playlist", { id });
      if (activePlaylistId === id) activePlaylistId = null;
      showToast(`Deleted "${name}"`);
    } catch (e) {
      error = String(e);
    }
  }

  async function addTo(playlistId, track) {
    try {
      playlists = await invoke("add_to_playlist", { playlistId, track });
      addMenuTrackId = null;
      showToast("Added to playlist");
      triggerMascot("Track saved to playlist nya~");
    } catch (e) {
      error = String(e);
    }
  }

  async function removeFrom(playlistId, trackId) {
    try {
      playlists = await invoke("remove_from_playlist", { playlistId, trackId });
      showToast("Removed from playlist");
    } catch (e) {
      error = String(e);
    }
  }

  function addToQueue(track) {
    if (!queue.some(t => t.id === track.id)) {
      queue = [...queue, track];
      showToast(`Queued: ${track.title.slice(0, 22)}`);
    } else {
      showToast("Already in queue");
    }
  }

  function clearQueue() {
    if (currentTrack) {
      queue = [currentTrack];
      queueIndex = 0;
    } else {
      queue = [];
      queueIndex = 0;
    }
    showToast("Queue cleared");
  }

  async function clearCache() {
    if (!confirm("Clear all locally cached songs?")) return;
    try {
      await invoke("clear_cache");
      await updateCacheStats();
      showToast("Cache cleaned");
    } catch (e) {
      error = String(e);
    }
  }

  $: activePlaylist = playlists.find((p) => p.id === activePlaylistId);
  $: currentMascotSrc = mascotExpressions.find(m => m.id === currentMascotMood)?.file || mascotExpressions[0].file;
</script>

<!-- Animated Background Particles Canvas -->
<canvas bind:this={particlesCanvas} class="ambient-particles-canvas"></canvas>

<!-- Top Cinematic Intro Loading Screen -->
{#if showIntroLoading}
  <div class="intro-loading-screen" out:fade={{ duration: 350 }}>
    <div class="intro-content">
      <div class="intro-logo">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2">
          <path d="M9 18V5l12-2v13"></path>
          <circle cx="6" cy="18" r="3"></circle>
          <circle cx="18" cy="16" r="3"></circle>
        </svg>
      </div>
      <h1 class="intro-title">PROSOUNDS</h1>
      <p class="intro-sub">Next-Gen Audio Experience</p>
      <div class="intro-bar-wrap">
        <div class="intro-bar-fill" style="width: {introProgress}%;"></div>
      </div>
    </div>
  </div>
{/if}

<!-- First-time Onboarding Modal -->
{#if showOnboarding}
  <div class="onboarding-overlay" transition:fade>
    <div class="onboarding-card" in:scale={{ duration: 250, start: 0.9 }}>
      <div class="onboarding-avatar">
        <img src="/mascot_cutouts/mascot_nya.png" alt="Mascot" />
      </div>
      <h2>Hi! What should I call you?</h2>
      <p>Personalize your ProSounds journey and companion.</p>
      <div class="onboarding-input-row">
        <input
          bind:value={tempNameInput}
          placeholder="Enter your name or nickname..."
          on:keydown={(e) => e.key === "Enter" && saveUserName()}
        />
        <button on:click={saveUserName}>Start Vibing</button>
      </div>
    </div>
  </div>
{/if}

<div
  class="app-wrapper theme-{theme}"
  style="
    --bg-opacity: {theme === 'custom' ? customBgDarkness : bgDarkness};
    --custom-accent: {customAccent};
    --reactive-scale: {1 + audioBassPulse};
  "
>
  {#if loading || resolvingId}
    <div class="top-loading-bar" in:fly={{ y: -10 }}></div>
  {/if}

  {#if toastMessage}
    <div class="toast-notification" transition:fly={{ y: 20 }}>
      {toastMessage}
    </div>
  {/if}

  <main>
    <!-- Top Header -->
    <header>
      <div class="brand">
        <div class="brand-logo">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <path d="M9 18V5l12-2v13"></path>
            <circle cx="6" cy="18" r="3"></circle>
            <circle cx="18" cy="16" r="3"></circle>
          </svg>
        </div>
        <div class="brand-text">
          <span class="logo-title">PROSOUNDS</span>
          <span class="logo-subtitle">{userName ? `Welcome, ${userName}` : 'Hi-Fi Audio'}</span>
        </div>
      </div>

      <!-- Search Input -->
      <div class="search-box">
        <svg class="search-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="11" cy="11" r="8"></circle>
          <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
        </svg>
        <input
          bind:this={searchInputEl}
          bind:value={query}
          on:keydown={(e) => e.key === "Enter" && doSearch()}
          placeholder="Search SoundCloud tracks, mixes, artists... (/)"
        />
        {#if query}
          <button class="clear-btn" on:click={() => { query = ""; searchInputEl?.focus(); }}>✕</button>
        {/if}
        <button class="search-submit" on:click={doSearch} disabled={loading}>
          {loading ? "Searching..." : "Search"}
        </button>
      </div>

      <!-- Nav Tabs -->
      <nav class="nav-tabs">
        <button class:active={activeTab === "recommended"} on:click={() => (activeTab = "recommended")}>
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"></polygon></svg>
          Recommended
        </button>
        <button class:active={activeTab === "search"} on:click={() => (activeTab = "search")}>
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"></circle><polygon points="10 8 16 12 10 16 10 8"></polygon></svg>
          Discover
        </button>
        <button class:active={activeTab === "queue"} on:click={() => (activeTab = "queue")}>
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="8" y1="6" x2="21" y2="6"></line><line x1="8" y1="12" x2="21" y2="12"></line><line x1="8" y1="18" x2="21" y2="18"></line><line x1="3" y1="6" x2="3.01" y2="6"></line><line x1="3" y1="12" x2="3.01" y2="12"></line><line x1="3" y1="18" x2="3.01" y2="18"></line></svg>
          Queue ({queue.length})
        </button>
        <button class:active={activeTab === "playlists"} on:click={() => (activeTab = "playlists")}>
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path></svg>
          Playlists
        </button>
        <button class:active={activeTab === "settings"} on:click={() => (activeTab = "settings")}>
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"></circle><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path></svg>
          Settings
        </button>
      </nav>
    </header>

    {#if error}
      <div class="error-banner" transition:fade>
        <span>Error: {error}</span>
        <button on:click={() => (error = "")}>✕</button>
      </div>
    {/if}

    <!-- Content Sections -->
    <div class="main-body">
      {#if activeTab === "recommended"}
        <!-- Recommended Curated Tab -->
        <section class="recommended-section" transition:fade>
          <div class="welcome-banner">
            <div>
              <h2>{userName ? `For You, ${userName}` : 'Recommended Tracks'}</h2>
              <p class="muted">Hand-curated listening sessions designed for focus, relaxation, and energy.</p>
            </div>
            <div class="status-pill">120 FPS Active</div>
          </div>

          <div class="curated-grid">
            {#each curatedPicks as pick}
              <div class="curated-card">
                <div class="curated-header">
                  <span class="genre-tag">{pick.genre}</span>
                  <button class="quick-play-btn" on:click={() => playCurated(pick)}>Play</button>
                </div>
                <div class="curated-meta">
                  <span class="curated-title">{pick.title}</span>
                  <span class="curated-artist">{pick.artist}</span>
                </div>
              </div>
            {/each}
          </div>
        </section>

      {:else if activeTab === "search"}
        <!-- Discover Search Results -->
        <section class="results-section" transition:fade>
          <div class="section-header">
            <h2>Search Results</h2>
            {#if results.length > 0}
              <span class="track-count">{results.length} tracks found</span>
            {/if}
          </div>

          {#if loading}
            <div class="grid-skeleton">
              {#each Array(6) as _}
                <div class="skeleton-card"></div>
              {/each}
            </div>
          {:else if results.length === 0}
            <div class="empty-state">
              <p>Type a song, artist, or vibe in the search bar above.</p>
            </div>
          {:else}
            <div class="tracks-grid">
              {#each results as track (track.id)}
                <div
                  class="track-card"
                  class:is-active={currentTrack?.id === track.id}
                  class:is-resolving={resolvingId === track.id}
                >
                  <div
                    class="card-thumb-wrap"
                    role="button"
                    tabindex="0"
                    on:click={() => playTrack(track, results)}
                    on:keydown={(e) => (e.key === "Enter" || e.key === " ") && playTrack(track, results)}
                  >
                    {#if track.thumbnail}
                      <img src={track.thumbnail} alt={track.title} class="card-thumb" loading="lazy" />
                    {:else}
                      <div class="placeholder-art">SoundCloud</div>
                    {/if}
                    <button class="play-overlay-btn" aria-label="Play">
                      {#if resolvingId === track.id}
                        <div class="mini-spinner"></div>
                      {:else if currentTrack?.id === track.id && isPlaying}
                        ❚❚
                      {:else}
                        ▶
                      {/if}
                    </button>
                    <span class="duration-badge">{fmt(track.duration)}</span>
                  </div>

                  <div class="card-details">
                    <span class="card-title" title={track.title}>{track.title}</span>
                    <span class="card-artist" title={track.artist}>{track.artist}</span>
                  </div>

                  <div class="card-actions">
                    <button class="btn-action" on:click|stopPropagation={() => addToQueue(track)}>
                      Queue
                    </button>
                    <button class="btn-action" on:click|stopPropagation={() => (addMenuTrackId = addMenuTrackId === track.id ? null : track.id)}>
                      + List
                    </button>
                  </div>

                  {#if addMenuTrackId === track.id}
                    <div class="playlist-dropdown" transition:fly={{ y: -8 }}>
                      <div class="dropdown-header">Add to Playlist</div>
                      {#if playlists.length === 0}
                        <p class="dropdown-empty">No playlists yet.</p>
                      {:else}
                        {#each playlists as pl (pl.id)}
                          <button class="dropdown-item" on:click|stopPropagation={() => addTo(pl.id, track)}>
                            <span>{pl.name}</span>
                            <span class="dropdown-count">{pl.tracks.length}</span>
                          </button>
                        {/each}
                      {/if}
                    </div>
                  {/if}
                </div>
              {/each}
            </div>
          {/if}
        </section>

      {:else if activeTab === "queue"}
        <!-- Queue View -->
        <section class="queue-section" transition:fade>
          <div class="section-header">
            <h2>Current Queue ({queue.length})</h2>
            {#if queue.length > 0}
              <button class="btn-pill" on:click={clearQueue}>Clear Queue</button>
            {/if}
          </div>

          {#if queue.length === 0}
            <div class="empty-state">
              <p>Queue is empty. Click "Queue" on any track to add it here.</p>
            </div>
          {:else}
            <div class="queue-list">
              {#each queue as track, idx (track.id + idx)}
                <div class="queue-item" class:is-current={queueIndex === idx}>
                  <span class="q-index">{idx + 1}</span>
                  <div class="q-thumb">
                    {#if track.thumbnail}
                      <img src={track.thumbnail} alt="" />
                    {:else}
                      ♪
                    {/if}
                  </div>
                  <div
                    class="q-info"
                    role="button"
                    tabindex="0"
                    on:click={() => { queueIndex = idx; playTrack(track, queue); }}
                    on:keydown={(e) => (e.key === "Enter" || e.key === " ") && (queueIndex = idx, playTrack(track, queue))}
                  >
                    <span class="q-title">{track.title}</span>
                    <span class="q-artist">{track.artist}</span>
                  </div>
                  <span class="q-dur">{fmt(track.duration)}</span>
                  <button class="q-remove" on:click={() => {
                    queue = queue.filter((_, i) => i !== idx);
                    if (queueIndex >= queue.length) queueIndex = Math.max(0, queue.length - 1);
                  }}>✕</button>
                </div>
              {/each}
            </div>
          {/if}
        </section>

      {:else if activeTab === "playlists"}
        <!-- Playlists View -->
        <section class="playlists-container" transition:fade>
          <aside class="playlist-sidebar">
            <div class="sidebar-header">
              <h3>Playlists</h3>
              <button class="btn-new-pl" on:click={() => (showNewPlaylist = !showNewPlaylist)}>+ New</button>
            </div>

            {#if showNewPlaylist}
              <div class="new-pl-input-row" transition:slide>
                <input
                  bind:value={newPlaylistName}
                  placeholder="Playlist name..."
                  on:keydown={(e) => e.key === "Enter" && createPlaylist()}
                />
                <button on:click={createPlaylist}>Save</button>
              </div>
            {/if}

            <div class="playlist-nav-list">
              {#each playlists as pl (pl.id)}
                <div class="pl-nav-item" class:selected={activePlaylistId === pl.id}>
                  <button class="pl-select-btn" on:click={() => (activePlaylistId = pl.id)}>
                    <span class="pl-name">{pl.name}</span>
                    <span class="pl-badge">{pl.tracks.length}</span>
                  </button>
                  <button class="pl-del-btn" on:click|stopPropagation={() => deletePlaylist(pl.id, pl.name)}>✕</button>
                </div>
              {/each}
            </div>
          </aside>

          <div class="playlist-content-view">
            {#if activePlaylist}
              <div class="pl-view-header">
                <div>
                  <h2>{activePlaylist.name}</h2>
                  <p class="muted">{activePlaylist.tracks.length} tracks</p>
                </div>
                {#if activePlaylist.tracks.length > 0}
                  <button class="btn-play-all" on:click={() => playTrack(activePlaylist.tracks[0], activePlaylist.tracks)}>
                    Play All
                  </button>
                {/if}
              </div>

              {#if activePlaylist.tracks.length === 0}
                <div class="empty-state">
                  <p>Playlist is empty. Add tracks from Discover.</p>
                </div>
              {:else}
                <div class="pl-track-list">
                  {#each activePlaylist.tracks as t, i (t.id)}
                    <div class="pl-track-row" class:is-playing={currentTrack?.id === t.id}>
                      <span class="track-num">{i + 1}</span>
                      <div
                        class="track-main"
                        role="button"
                        tabindex="0"
                        on:click={() => playTrack(t, activePlaylist.tracks)}
                        on:keydown={(e) => (e.key === "Enter" || e.key === " ") && playTrack(t, activePlaylist.tracks)}
                      >
                        <span class="track-row-title">{t.title}</span>
                        <span class="track-row-artist">{t.artist}</span>
                      </div>
                      <span class="track-row-dur">{fmt(t.duration)}</span>
                      <button class="track-del-btn" on:click={() => removeFrom(activePlaylist.id, t.id)}>✕</button>
                    </div>
                  {/each}
                </div>
              {/if}
            {:else}
              <div class="empty-state">
                <p>Select or create a playlist from the sidebar.</p>
              </div>
            {/if}
          </div>
        </section>

      {:else if activeTab === "settings"}
        <!-- Settings & 5+ Themes + Custom Theme Creator -->
        <section class="settings-view" transition:fade>
          <h2>Settings & Themes</h2>

          <div class="settings-cards">
            <!-- User Profile -->
            <div class="settings-box">
              <h3>Profile</h3>
              <div class="setting-row">
                <span>Display Name</span>
                <div class="inline-edit">
                  <input bind:value={userName} on:change={() => localStorage.setItem("prosounds_user", userName)} />
                </div>
              </div>
            </div>

            <!-- Themes (5 Presets + Custom Creator) -->
            <div class="settings-box">
              <h3>Themes (5 Epic Presets + Custom)</h3>
              <div class="theme-buttons-grid">
                <button class="theme-btn" class:active={theme === 'midnight'} on:click={() => applyTheme('midnight')}>
                  <span class="color-dot" style="background: #a6c1ee;"></span>
                  Midnight Velvet
                </button>
                <button class="theme-btn" class:active={theme === 'cyberpunk'} on:click={() => applyTheme('cyberpunk')}>
                  <span class="color-dot" style="background: #00f0ff;"></span>
                  Cyberpunk Cyan
                </button>
                <button class="theme-btn" class:active={theme === 'sakura'} on:click={() => applyTheme('sakura')}>
                  <span class="color-dot" style="background: #ff7597;"></span>
                  Sakura Dream
                </button>
                <button class="theme-btn" class:active={theme === 'sunset'} on:click={() => applyTheme('sunset')}>
                  <span class="color-dot" style="background: #ff8c42;"></span>
                  Sunset Horizon
                </button>
                <button class="theme-btn" class:active={theme === 'emerald'} on:click={() => applyTheme('emerald')}>
                  <span class="color-dot" style="background: #10b981;"></span>
                  Emerald Matrix
                </button>
                <button class="theme-btn" class:active={theme === 'custom'} on:click={() => applyTheme('custom')}>
                  <span class="color-dot" style="background: {customAccent};"></span>
                  Custom Theme
                </button>
              </div>

              {#if theme === "custom"}
                <div class="custom-theme-panel" transition:slide>
                  <h4>Design Your Custom Theme</h4>
                  <div class="setting-row">
                    <span>Accent Color</span>
                    <input type="color" bind:value={customAccent} on:change={saveCustomTheme} />
                  </div>
                  <div class="setting-row">
                    <span>Background Glass Tint</span>
                    <input type="range" min="0.3" max="0.95" step="0.05" bind:value={customBgDarkness} on:change={saveCustomTheme} />
                  </div>
                </div>
              {/if}
            </div>

            <!-- Audio Enhancer & Auto Normalization -->
            <div class="settings-box">
              <h3>Audio Enhancements</h3>
              <div class="setting-row">
                <div>
                  <span class="strong">Smart Volume Normalizer (Auto AGC)</span>
                  <p class="muted sub">Maintains consistent loudness and smooth crossfades between songs.</p>
                </div>
                <input type="checkbox" bind:checked={autoNormalizer} />
              </div>
            </div>

            <!-- Living Background & Mascot -->
            <div class="settings-box">
              <h3>Living Background & Companion</h3>
              <div class="setting-row">
                <span>Living Ambient Particles (120 FPS)</span>
                <input type="checkbox" bind:checked={animatedParticles} />
              </div>

              <div class="setting-row">
                <span>Companion Mascot</span>
                <input type="checkbox" bind:checked={showMascot} />
              </div>

              <div class="setting-row">
                <span>Mascot Voicelines & Nya Chimes</span>
                <button class="btn-pill" on:click={handleMascotClick}>Test Nya Chime</button>
              </div>
            </div>

            <!-- Audio Cache Management -->
            <div class="settings-box">
              <h3>Local Cache</h3>
              <div class="cache-stat-display">
                <span class="stat-num">{formatBytes(cacheSizeBytes)}</span>
                <span class="stat-label">Instant Offline Storage</span>
              </div>
              <button class="btn-clear-cache" on:click={clearCache}>Clear Audio Cache</button>
            </div>
          </div>
        </section>
      {/if}
    </div>

    <!-- Bottom Audio Player Bar -->
    <footer>
      <div class="now-playing-col">
        <div class="cover-art-container" class:playing={isPlaying}>
          {#if currentTrack?.thumbnail}
            <img src={currentTrack.thumbnail} alt="Cover" class="cover-img" />
          {:else}
            <div class="cover-placeholder">♪</div>
          {/if}
        </div>
        <div class="track-meta">
          <span class="np-title" title={currentTrack?.title || "No track selected"}>
            {currentTrack?.title || "Ready to play"}
          </span>
          <span class="np-artist">
            {currentTrack?.artist || "ProSounds"}
          </span>
        </div>
      </div>

      <!-- Center Controls -->
      <div class="player-center-col">
        <div class="transport-controls">
          <button class="ctl-btn" class:active-state={shuffle} on:click={() => (shuffle = !shuffle)} title="Shuffle (S)">
            ⇄
          </button>
          <button class="ctl-btn" on:click={playPrevTrack} title="Previous (P)">
            ◀◀
          </button>
          <button class="ctl-btn" on:click={() => seekRelative(-5)} title="Back 5s">
            -5s
          </button>
          <button class="ctl-btn play-main-btn" on:click={togglePlay} title="Play/Pause (Space)">
            {#if isPlaying}
              ❚❚
            {:else}
              ▶
            {/if}
          </button>
          <button class="ctl-btn" on:click={() => seekRelative(5)} title="Forward 5s">
            +5s
          </button>
          <button class="ctl-btn" on:click={playNextTrack} title="Next (N)">
            ▶▶
          </button>
          <button class="ctl-btn" class:active-state={repeatMode !== "off"} on:click={cycleRepeat} title="Repeat (L)">
            {repeatMode === "one" ? "1" : repeatMode === "all" ? "All" : "Off"}
          </button>
        </div>

        <!-- Progress Seekbar -->
        <div class="progress-container">
          <span class="timestamp">{fmt(currentTime)}</span>
          <div
            class="seek-track"
            on:mousedown={handleSeekMouseDown}
            on:mousemove={handleSeekMouseMove}
            on:mouseup={handleSeekMouseUp}
            on:mouseleave={() => (hoverTime = null)}
            role="slider"
            tabindex="0"
            aria-valuenow={currentTime}
            aria-valuemin="0"
            aria-valuemax={duration}
          >
            <div class="seek-filled" style="width: {duration ? (currentTime / duration) * 100 : 0}%;">
              <div class="seek-thumb"></div>
            </div>
            {#if hoverTime !== null}
              <div class="hover-tooltip" style="left: {hoverPct * 100}%;">
                {fmt(hoverTime)}
              </div>
            {/if}
          </div>
          <span class="timestamp duration">{fmt(duration)}</span>
        </div>
      </div>

      <!-- Right Visualizer & Volume -->
      <div class="player-right-col">
        <div class="visualizer-wrapper">
          <canvas bind:this={visualizerCanvas} width="120" height="34" class="spectrum-canvas"></canvas>
        </div>

        <div class="volume-control">
          <button class="mute-btn" on:click={toggleMute} title="Mute/Unmute (M)">
            {#if isMuted || volume === 0}
              [Muted]
            {:else}
              Vol
            {/if}
          </button>
          <input
            type="range"
            min="0"
            max="1"
            step="0.01"
            bind:value={volume}
            on:input={setVolumeFromInput}
            class="vol-slider"
          />
        </div>
      </div>
    </footer>
  </main>

  <!-- Mascot Companion: Rotates Expression on Every Click + Anime Voicelines -->
  {#if showMascot}
    <div
      class="mascot-dock"
      class:mascot-groove={isPlaying}
      class:spin={mascotSpinning}
      role="button"
      tabindex="0"
      on:click={handleMascotClick}
      on:keydown={(e) => (e.key === "Enter" || e.key === " ") && handleMascotClick()}
      title="Click me for cute expressions and nya chimes!"
      transition:fly={{ y: 30 }}
    >
      {#if showBubble}
        <div class="speech-bubble" transition:fade={{ duration: 150 }}>
          <p>{mascotMessage}</p>
        </div>
      {/if}

      <div class="mascot-avatar-wrap">
        <!-- Render cleanly cut out mascot expression -->
        <img
          src={currentMascotSrc}
          alt="ProSounds Anime Mascot"
          class="mascot-img"
        />
        <div class="mascot-mood-badge">{currentMascotMood.toUpperCase()}</div>
      </div>
    </div>
  {/if}
</div>

<style>
  :global(html), :global(body) {
    margin: 0;
    padding: 0;
    height: 100%;
    overflow: hidden;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Inter", sans-serif;
    font-size: 13.5px;
    background: #08080c;
    color: #ffffff;
    user-select: none;
  }

  /* Themes */
  .theme-midnight {
    --accent: #a6c1ee;
    --accent-glow: rgba(166, 193, 238, 0.4);
    --panel-bg: rgba(14, 14, 20, 0.75);
    --card-bg: rgba(20, 20, 30, 0.6);
    --border: rgba(255, 255, 255, 0.12);
    --border-hover: rgba(166, 193, 238, 0.5);
  }
  .theme-cyberpunk {
    --accent: #00f0ff;
    --accent-glow: rgba(0, 240, 255, 0.45);
    --panel-bg: rgba(10, 8, 20, 0.78);
    --card-bg: rgba(18, 14, 32, 0.65);
    --border: rgba(0, 240, 255, 0.25);
    --border-hover: rgba(0, 240, 255, 0.6);
  }
  .theme-sakura {
    --accent: #ff7597;
    --accent-glow: rgba(255, 117, 151, 0.45);
    --panel-bg: rgba(24, 14, 20, 0.78);
    --card-bg: rgba(34, 18, 26, 0.65);
    --border: rgba(255, 117, 151, 0.25);
    --border-hover: rgba(255, 117, 151, 0.6);
  }
  .theme-sunset {
    --accent: #ff8c42;
    --accent-glow: rgba(255, 140, 66, 0.45);
    --panel-bg: rgba(20, 12, 12, 0.78);
    --card-bg: rgba(30, 18, 18, 0.65);
    --border: rgba(255, 140, 66, 0.22);
    --border-hover: rgba(255, 180, 80, 0.6);
  }
  .theme-emerald {
    --accent: #10b981;
    --accent-glow: rgba(16, 185, 129, 0.45);
    --panel-bg: rgba(8, 20, 16, 0.78);
    --card-bg: rgba(12, 30, 22, 0.65);
    --border: rgba(16, 185, 129, 0.22);
    --border-hover: rgba(52, 211, 153, 0.6);
  }
  .theme-custom {
    --accent: var(--custom-accent, #a6c1ee);
    --accent-glow: rgba(255, 255, 255, 0.25);
    --panel-bg: rgba(14, 14, 20, 0.78);
    --card-bg: rgba(20, 20, 30, 0.65);
    --border: rgba(255, 255, 255, 0.15);
    --border-hover: var(--custom-accent, #a6c1ee);
  }

  /* Crisp, Non-Blurry HD Background */
  .app-wrapper {
    height: 100vh;
    display: flex;
    flex-direction: column;
    position: relative;
    overflow: hidden;
    background:
      linear-gradient(rgba(8, 8, 14, var(--bg-opacity, 0.65)), rgba(8, 8, 14, var(--bg-opacity, 0.65))),
      url('/bg.png') center / cover no-repeat fixed #08080c;
    transition: background 0.3s ease;
  }

  /* Ambient Particle Canvas (120 FPS capable) */
  .ambient-particles-canvas {
    position: absolute;
    top: 0; left: 0;
    width: 100%; height: 100%;
    pointer-events: none;
    z-index: 0;
  }

  /* Intro Loading Screen */
  .intro-loading-screen {
    position: fixed;
    top: 0; left: 0; right: 0; bottom: 0;
    z-index: 9999;
    background: #08080c;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .intro-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    max-width: 320px;
  }
  .intro-logo {
    width: 48px;
    height: 48px;
    border-radius: 12px;
    background: linear-gradient(135deg, #a6c1ee 0%, #00f0ff 100%);
    display: flex;
    align-items: center;
    justify-content: center;
    color: #08080c;
    margin-bottom: 14px;
    box-shadow: 0 0 24px rgba(0, 240, 255, 0.5);
  }
  .intro-logo svg { width: 26px; height: 26px; }
  .intro-title { font-size: 22px; font-weight: 800; letter-spacing: 2px; margin: 0 0 4px; color: #fff; }
  .intro-sub { font-size: 11px; color: rgba(255,255,255,0.45); margin: 0 0 20px; letter-spacing: 0.5px; }
  .intro-bar-wrap { width: 180px; height: 3px; background: rgba(255,255,255,0.1); border-radius: 2px; overflow: hidden; }
  .intro-bar-fill { height: 100%; background: #00f0ff; transition: width 0.1s linear; }

  /* First-Time Onboarding Modal */
  .onboarding-overlay {
    position: fixed;
    top: 0; left: 0; right: 0; bottom: 0;
    background: rgba(0, 0, 0, 0.85);
    backdrop-filter: blur(20px);
    z-index: 8000;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .onboarding-card {
    background: var(--panel-bg);
    border: 1px solid var(--accent);
    border-radius: 16px;
    padding: 32px 36px;
    max-width: 420px;
    width: 90%;
    text-align: center;
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.7);
  }
  .onboarding-avatar {
    width: 140px;
    height: 70px;
    margin: 0 auto 12px;
  }
  .onboarding-avatar img { width: 100%; height: 100%; object-fit: contain; }
  .onboarding-card h2 { font-size: 20px; margin: 0 0 6px; }
  .onboarding-card p { font-size: 13px; color: rgba(255,255,255,0.55); margin: 0 0 22px; }
  .onboarding-input-row { display: flex; gap: 8px; }
  .onboarding-input-row input {
    flex: 1;
    background: rgba(0,0,0,0.45);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 10px 14px;
    color: #fff;
    font-size: 13.5px;
    outline: none;
  }
  .onboarding-input-row button {
    background: var(--accent);
    color: #08080c;
    border: none;
    font-weight: 600;
    padding: 0 18px;
    border-radius: 8px;
    cursor: pointer;
  }

  .top-loading-bar {
    position: absolute;
    top: 0; left: 0;
    height: 3px;
    background: var(--accent);
    box-shadow: 0 0 10px var(--accent);
    animation: barScan 1.6s ease-in-out infinite;
    z-index: 100;
  }
  @keyframes barScan {
    0% { width: 0; left: 0; }
    50% { width: 70%; left: 30%; }
    100% { width: 0; left: 100%; }
  }

  .toast-notification {
    position: fixed;
    top: 20px;
    right: 24px;
    z-index: 999;
    background: var(--panel-bg);
    backdrop-filter: blur(16px);
    border: 1px solid var(--accent);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.6);
    padding: 9px 16px;
    border-radius: 8px;
    font-size: 12.5px;
    font-weight: 500;
    color: #fff;
  }

  main {
    position: relative;
    z-index: 1;
    display: flex;
    flex-direction: column;
    height: 100vh;
  }

  /* Header */
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 24px;
    background: var(--panel-bg);
    backdrop-filter: blur(16px);
    border-bottom: 1px solid var(--border);
    gap: 16px;
  }

  .brand { display: flex; align-items: center; gap: 10px; }
  .brand-logo {
    width: 32px;
    height: 32px;
    border-radius: 8px;
    background: var(--accent);
    display: flex;
    align-items: center;
    justify-content: center;
    color: #08080c;
  }
  .brand-logo svg { width: 18px; height: 18px; }
  .brand-text { display: flex; flex-direction: column; }
  .logo-title { font-size: 15px; font-weight: 700; letter-spacing: 1px; color: #fff; }
  .logo-subtitle { font-size: 10px; color: rgba(255, 255, 255, 0.5); }

  .search-box {
    display: flex;
    align-items: center;
    position: relative;
    flex: 1;
    max-width: 440px;
  }
  .search-icon {
    position: absolute;
    left: 12px;
    width: 15px;
    height: 15px;
    color: rgba(255, 255, 255, 0.45);
    pointer-events: none;
  }
  .search-box input {
    width: 100%;
    padding: 8px 85px 8px 36px;
    background: rgba(0, 0, 0, 0.4);
    border: 1px solid var(--border);
    border-radius: 8px;
    color: #ffffff;
    font-size: 13px;
    outline: none;
    transition: border-color 0.15s ease;
  }
  .search-box input:focus {
    border-color: var(--accent);
    background: rgba(0, 0, 0, 0.6);
  }
  .clear-btn {
    position: absolute;
    right: 70px;
    background: transparent;
    border: none;
    color: rgba(255, 255, 255, 0.45);
    cursor: pointer;
    font-size: 12px;
  }
  .search-submit {
    position: absolute;
    right: 5px;
    padding: 5px 12px;
    background: var(--accent);
    color: #08080c;
    font-weight: 600;
    font-size: 11.5px;
    border: none;
    border-radius: 6px;
    cursor: pointer;
  }

  .nav-tabs { display: flex; align-items: center; gap: 4px; }
  .nav-tabs button {
    display: flex;
    align-items: center;
    gap: 6px;
    background: transparent;
    border: 1px solid transparent;
    color: rgba(255, 255, 255, 0.7);
    padding: 7px 12px;
    border-radius: 8px;
    cursor: pointer;
    font-size: 12.5px;
    font-weight: 500;
  }
  .nav-tabs button svg { width: 14px; height: 14px; }
  .nav-tabs button:hover { background: rgba(255, 255, 255, 0.08); color: #fff; }
  .nav-tabs button.active {
    background: rgba(255, 255, 255, 0.12);
    border-color: var(--border);
    color: var(--accent);
  }

  .error-banner {
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: rgba(220, 38, 38, 0.25);
    border-left: 3px solid #ef4444;
    padding: 8px 20px;
    font-size: 12.5px;
  }
  .error-banner button { background: transparent; border: none; color: #fff; cursor: pointer; }

  /* Body Content */
  .main-body { flex: 1; overflow-y: auto; padding: 18px 26px; }

  /* Recommended Section */
  .recommended-section { max-width: 960px; margin: 0 auto; }
  .welcome-banner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 24px;
    padding-bottom: 14px;
    border-bottom: 1px solid var(--border);
  }
  .welcome-banner h2 { font-size: 20px; margin: 0 0 4px; }
  .status-pill {
    background: rgba(16, 185, 129, 0.15);
    border: 1px solid #10b981;
    color: #34d399;
    font-size: 11px;
    font-weight: 600;
    padding: 4px 10px;
    border-radius: 20px;
  }

  .curated-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
    gap: 16px;
  }
  .curated-card {
    background: var(--card-bg);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 14px;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    min-height: 110px;
    backdrop-filter: blur(14px);
    transition: transform 0.15s ease, border-color 0.15s ease;
  }
  .curated-card:hover {
    border-color: var(--accent);
    transform: translateY(-2px);
  }
  .curated-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 10px; }
  .genre-tag {
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    background: rgba(255, 255, 255, 0.1);
    padding: 2px 7px;
    border-radius: 4px;
    color: var(--accent);
  }
  .quick-play-btn {
    background: var(--accent);
    color: #08080c;
    border: none;
    font-weight: 600;
    font-size: 11px;
    padding: 3px 10px;
    border-radius: 5px;
    cursor: pointer;
  }
  .curated-meta { display: flex; flex-direction: column; }
  .curated-title { font-weight: 600; font-size: 13px; margin-bottom: 2px; }
  .curated-artist { font-size: 11.5px; color: rgba(255,255,255,0.5); }

  /* Tracks Grid */
  .section-header { display: flex; align-items: baseline; justify-content: space-between; margin-bottom: 14px; }
  .section-header h2 { font-size: 17px; font-weight: 600; margin: 0; }
  .track-count { font-size: 11.5px; color: rgba(255, 255, 255, 0.45); }

  .tracks-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(190px, 1fr));
    gap: 16px;
  }
  .track-card {
    background: var(--card-bg);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 10px;
    display: flex;
    flex-direction: column;
    backdrop-filter: blur(14px);
    position: relative;
    transition: transform 0.15s ease, border-color 0.15s ease;
  }
  .track-card:hover { border-color: var(--border-hover); transform: translateY(-3px); }
  .track-card.is-active { border-color: var(--accent); }

  .card-thumb-wrap {
    position: relative;
    width: 100%;
    padding-top: 100%;
    border-radius: 8px;
    overflow: hidden;
    background: #12121a;
    cursor: pointer;
  }
  .card-thumb { position: absolute; top: 0; left: 0; width: 100%; height: 100%; object-fit: cover; }
  .placeholder-art {
    position: absolute;
    top: 0; left: 0; width: 100%; height: 100%;
    display: flex; align-items: center; justify-content: center;
    font-size: 12px; color: rgba(255,255,255,0.3);
  }

  .play-overlay-btn {
    position: absolute;
    bottom: 10px;
    right: 10px;
    width: 36px;
    height: 36px;
    border-radius: 50%;
    background: var(--accent);
    color: #08080c;
    border: none;
    font-size: 14px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    opacity: 0;
    transform: translateY(4px);
    transition: all 0.15s ease;
  }
  .track-card:hover .play-overlay-btn,
  .track-card.is-active .play-overlay-btn { opacity: 1; transform: translateY(0); }

  .mini-spinner {
    width: 12px;
    height: 12px;
    border: 2px solid #08080c;
    border-top-color: transparent;
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }

  .duration-badge {
    position: absolute;
    bottom: 8px;
    left: 8px;
    background: rgba(0, 0, 0, 0.75);
    font-size: 10.5px;
    font-weight: 500;
    padding: 2px 6px;
    border-radius: 4px;
  }

  .card-details { margin-top: 8px; display: flex; flex-direction: column; gap: 2px; }
  .card-title { font-weight: 500; font-size: 13px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .card-artist { font-size: 11.5px; color: rgba(255, 255, 255, 0.5); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .card-actions { margin-top: 8px; display: flex; gap: 6px; }
  .btn-action {
    flex: 1;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid var(--border);
    color: #ffffff;
    font-size: 11px;
    padding: 5px 0;
    border-radius: 6px;
    cursor: pointer;
  }
  .btn-action:hover { background: rgba(255, 255, 255, 0.15); }

  .playlist-dropdown {
    position: absolute;
    top: calc(100% - 6px);
    right: 10px;
    z-index: 50;
    background: var(--panel-bg);
    backdrop-filter: blur(20px);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 6px;
    min-width: 160px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.6);
  }
  .dropdown-header { font-size: 10.5px; font-weight: 600; color: rgba(255, 255, 255, 0.45); padding: 4px 6px; text-transform: uppercase; }
  .dropdown-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 6px 8px;
    background: transparent;
    border: none;
    color: #ffffff;
    border-radius: 4px;
    font-size: 12px;
    cursor: pointer;
    text-align: left;
  }
  .dropdown-item:hover { background: rgba(255, 255, 255, 0.1); color: var(--accent); }
  .dropdown-count { font-size: 10px; color: rgba(255, 255, 255, 0.4); }
  .dropdown-empty { font-size: 11.5px; color: rgba(255, 255, 255, 0.45); padding: 4px 6px; margin: 0; }

  .grid-skeleton { display: grid; grid-template-columns: repeat(auto-fill, minmax(190px, 1fr)); gap: 16px; }
  .skeleton-card { height: 240px; background: rgba(255, 255, 255, 0.04); border-radius: 12px; }
  .empty-state { padding: 50px 20px; text-align: center; color: rgba(255, 255, 255, 0.5); }

  /* Queue */
  .queue-section { max-width: 860px; margin: 0 auto; }
  .btn-pill {
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid var(--border);
    color: #ffffff;
    padding: 5px 12px;
    border-radius: 16px;
    cursor: pointer;
    font-size: 11.5px;
  }
  .queue-list { display: flex; flex-direction: column; gap: 5px; }
  .queue-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 12px;
    background: var(--card-bg);
    border: 1px solid var(--border);
    border-radius: 8px;
  }
  .queue-item.is-current { border-color: var(--accent); }
  .q-index { font-size: 11.5px; color: rgba(255,255,255,0.4); width: 18px; }
  .q-thumb { width: 34px; height: 34px; border-radius: 6px; overflow: hidden; background: #1a1a24; flex-shrink: 0; }
  .q-thumb img { width: 100%; height: 100%; object-fit: cover; }
  .q-info { flex: 1; display: flex; flex-direction: column; cursor: pointer; overflow: hidden; }
  .q-title { font-weight: 500; font-size: 12.5px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .q-artist { font-size: 11px; color: rgba(255,255,255,0.5); }
  .q-dur { font-size: 11.5px; color: rgba(255,255,255,0.5); }
  .q-remove { background: transparent; border: none; color: rgba(255,255,255,0.4); cursor: pointer; padding: 4px; }
  .q-remove:hover { color: #ef4444; }

  /* Playlists */
  .playlists-container { display: grid; grid-template-columns: 260px 1fr; gap: 20px; height: 100%; }
  .playlist-sidebar {
    background: var(--card-bg);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 14px;
    backdrop-filter: blur(14px);
    display: flex;
    flex-direction: column;
  }
  .sidebar-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 10px; }
  .sidebar-header h3 { margin: 0; font-size: 14px; }
  .btn-new-pl { background: var(--accent); color: #08080c; border: none; font-weight: 600; font-size: 11px; padding: 3px 8px; border-radius: 5px; cursor: pointer; }
  .new-pl-input-row { display: flex; gap: 5px; margin-bottom: 10px; }
  .new-pl-input-row input { flex: 1; background: rgba(0,0,0,0.4); border: 1px solid var(--border); border-radius: 6px; padding: 5px 8px; color: #fff; font-size: 12px; }
  .new-pl-input-row button { background: rgba(255,255,255,0.1); border: 1px solid var(--border); color: #fff; border-radius: 6px; padding: 5px 8px; cursor: pointer; }
  .playlist-nav-list { display: flex; flex-direction: column; gap: 4px; overflow-y: auto; }
  .pl-nav-item { display: flex; align-items: center; border-radius: 6px; background: transparent; }
  .pl-nav-item:hover { background: rgba(255, 255, 255, 0.08); }
  .pl-nav-item.selected { background: rgba(255, 255, 255, 0.12); border: 1px solid var(--border); }
  .pl-select-btn { flex: 1; display: flex; align-items: center; justify-content: space-between; background: transparent; border: none; color: #ffffff; padding: 8px 10px; font-size: 12.5px; cursor: pointer; text-align: left; }
  .pl-badge { background: rgba(255,255,255,0.1); padding: 2px 6px; border-radius: 8px; font-size: 10.5px; }
  .pl-del-btn { background: transparent; border: none; cursor: pointer; padding: 4px 6px; opacity: 0.4; color: #fff; }
  .pl-del-btn:hover { opacity: 1; color: #ef4444; }

  .playlist-content-view { background: var(--card-bg); border: 1px solid var(--border); border-radius: 12px; padding: 20px; backdrop-filter: blur(14px); overflow-y: auto; }
  .pl-view-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px; padding-bottom: 10px; border-bottom: 1px solid var(--border); }
  .pl-view-header h2 { margin: 0 0 2px; font-size: 18px; }
  .btn-play-all { background: var(--accent); color: #08080c; font-weight: 600; border: none; padding: 7px 16px; border-radius: 6px; cursor: pointer; }
  .pl-track-list { display: flex; flex-direction: column; gap: 4px; }
  .pl-track-row { display: flex; align-items: center; gap: 12px; padding: 8px 12px; border-radius: 6px; }
  .pl-track-row:hover { background: rgba(255, 255, 255, 0.08); }
  .pl-track-row.is-playing { background: rgba(166, 193, 238, 0.12); color: var(--accent); }
  .track-num { width: 18px; font-size: 11.5px; color: rgba(255,255,255,0.4); }
  .track-main { flex: 1; display: flex; flex-direction: column; cursor: pointer; }
  .track-row-title { font-weight: 500; font-size: 13px; }
  .track-row-artist { font-size: 11.5px; color: rgba(255,255,255,0.5); }
  .track-row-dur { font-size: 11.5px; color: rgba(255,255,255,0.5); }
  .track-del-btn { background: transparent; border: none; color: rgba(255,255,255,0.4); cursor: pointer; }
  .track-del-btn:hover { color: #ef4444; }

  /* Settings & Themes */
  .settings-view { max-width: 800px; margin: 0 auto; }
  .settings-view h2 { margin: 0 0 20px; font-size: 18px; }
  .settings-cards { display: flex; flex-direction: column; gap: 16px; }
  .settings-box { background: var(--card-bg); border: 1px solid var(--border); border-radius: 12px; padding: 18px; backdrop-filter: blur(14px); }
  .settings-box h3 { margin: 0 0 12px; font-size: 14px; color: var(--accent); }
  .setting-row { display: flex; align-items: center; justify-content: space-between; padding: 8px 0; border-bottom: 1px solid rgba(255,255,255,0.06); }
  .setting-row:last-child { border-bottom: none; }
  .inline-edit input { background: rgba(0,0,0,0.4); border: 1px solid var(--border); color: #fff; padding: 6px 10px; border-radius: 6px; }

  .theme-buttons-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 10px;
    margin-bottom: 12px;
  }
  .theme-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid var(--border);
    color: #fff;
    padding: 10px 12px;
    border-radius: 8px;
    cursor: pointer;
    font-size: 12px;
    transition: all 0.15s ease;
  }
  .theme-btn:hover { border-color: var(--accent); }
  .theme-btn.active { border-color: var(--accent); background: rgba(255,255,255,0.12); font-weight: 600; }
  .color-dot { width: 10px; height: 10px; border-radius: 50%; display: inline-block; }

  .custom-theme-panel {
    background: rgba(0,0,0,0.3);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 14px;
    margin-top: 10px;
  }
  .custom-theme-panel h4 { margin: 0 0 10px; font-size: 13px; }

  .cache-stat-display { margin: 10px 0; }
  .stat-num { font-size: 24px; font-weight: 700; color: var(--accent); }
  .stat-label { font-size: 11px; color: rgba(255,255,255,0.5); display: block; }
  .btn-clear-cache { background: rgba(255, 255, 255, 0.08); border: 1px solid var(--border); color: #fff; padding: 6px 14px; border-radius: 6px; cursor: pointer; font-size: 12px; }

  /* Player Footer */
  footer {
    height: 78px;
    background: var(--panel-bg);
    backdrop-filter: blur(20px);
    border-top: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 24px;
    gap: 16px;
    box-sizing: border-box;
    z-index: 40;
  }

  .now-playing-col { display: flex; align-items: center; gap: 12px; width: 250px; min-width: 220px; }
  .cover-art-container {
    width: 46px; height: 46px; border-radius: 6px; overflow: hidden;
    background: #14141e; border: 1px solid var(--border); flex-shrink: 0;
    display: flex; align-items: center; justify-content: center;
  }
  .cover-img { width: 100%; height: 100%; object-fit: cover; }
  .cover-placeholder { font-size: 16px; color: rgba(255,255,255,0.3); }

  .track-meta { display: flex; flex-direction: column; overflow: hidden; }
  .np-title { font-weight: 500; font-size: 13px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; color: #fff; }
  .np-artist { font-size: 11px; color: rgba(255, 255, 255, 0.5); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }

  .player-center-col { flex: 1; max-width: 560px; display: flex; flex-direction: column; align-items: center; gap: 4px; }
  .transport-controls { display: flex; align-items: center; gap: 10px; }
  .ctl-btn { background: transparent; border: none; color: rgba(255, 255, 255, 0.8); font-size: 12px; cursor: pointer; padding: 4px 8px; border-radius: 4px; }
  .ctl-btn:hover { color: #fff; background: rgba(255, 255, 255, 0.08); }
  .ctl-btn.active-state { color: var(--accent); }

  .play-main-btn {
    width: 34px; height: 34px; background: #ffffff; color: #08080c; border-radius: 50%;
    font-size: 13px; display: flex; align-items: center; justify-content: center; padding: 0;
  }
  .play-main-btn:hover { background: var(--accent); color: #08080c; }

  .progress-container { width: 100%; display: flex; align-items: center; gap: 8px; }
  .timestamp { font-size: 10.5px; font-variant-numeric: tabular-nums; color: rgba(255, 255, 255, 0.45); width: 32px; }
  .timestamp.duration { text-align: right; }
  .seek-track { flex: 1; height: 5px; background: rgba(255, 255, 255, 0.12); border-radius: 3px; position: relative; cursor: pointer; }
  .seek-filled { height: 100%; background: var(--accent); border-radius: 3px; position: relative; }
  .seek-thumb { position: absolute; right: -4px; top: 50%; transform: translateY(-50%); width: 10px; height: 10px; background: #fff; border-radius: 50%; opacity: 0; }
  .seek-track:hover .seek-thumb { opacity: 1; }

  .hover-tooltip {
    position: absolute; bottom: 12px; transform: translateX(-50%);
    background: #111; border: 1px solid var(--border); padding: 2px 5px; border-radius: 4px;
    font-size: 10px; pointer-events: none; color: #fff;
  }

  .player-right-col { display: flex; align-items: center; gap: 16px; width: 250px; justify-content: flex-end; }
  .visualizer-wrapper { width: 120px; height: 34px; }
  .spectrum-canvas { width: 120px; height: 34px; }

  .volume-control { display: flex; align-items: center; gap: 6px; }
  .mute-btn { background: transparent; border: none; color: rgba(255, 255, 255, 0.6); font-size: 11px; cursor: pointer; }
  .vol-slider { width: 70px; accent-color: var(--accent); }

  /* Cutout Mascot Companion Dock (Cropped Horizontal Bust) */
  .mascot-dock {
    position: fixed;
    bottom: 88px;
    right: 24px;
    z-index: 60;
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    cursor: pointer;
  }
  .mascot-avatar-wrap {
    position: relative;
    width: 140px;
    height: 75px;
    transition: transform 0.2s cubic-bezier(0.2, 0, 0, 1);
  }
  .mascot-img {
    width: 100%;
    height: 100%;
    object-fit: contain;
    filter: drop-shadow(0 6px 14px rgba(0, 0, 0, 0.6));
  }
  .mascot-dock:hover .mascot-img { transform: scale(1.06); }
  .mascot-groove .mascot-img { animation: groove 1.8s ease-in-out infinite alternate; }
  @keyframes groove {
    0% { transform: translateY(0px) rotate(0deg); }
    50% { transform: translateY(-4px) rotate(1deg); }
    100% { transform: translateY(0px) rotate(-1deg); }
  }
  .mascot-dock.spin .mascot-img { animation: spinJump 0.5s ease-out; }
  @keyframes spinJump {
    0% { transform: scale(1); }
    50% { transform: scale(1.15) translateY(-8px); }
    100% { transform: scale(1); }
  }

  .mascot-mood-badge {
    position: absolute;
    bottom: -4px;
    right: 6px;
    background: rgba(0, 0, 0, 0.8);
    border: 1px solid var(--border);
    font-size: 8px;
    font-weight: 700;
    letter-spacing: 0.5px;
    padding: 1px 4px;
    border-radius: 3px;
    color: var(--accent);
  }

  .speech-bubble {
    background: var(--panel-bg);
    backdrop-filter: blur(16px);
    border: 1px solid var(--accent);
    padding: 8px 12px;
    border-radius: 8px;
    margin-bottom: 6px;
    max-width: 220px;
    box-shadow: 0 4px 18px rgba(0, 0, 0, 0.5);
  }
  .speech-bubble p { margin: 0; font-size: 11.5px; line-height: 1.35; color: #fff; }
</style>
