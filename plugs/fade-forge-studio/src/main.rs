use js_sys::Promise;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use yew::prelude::*;

#[wasm_bindgen(inline_js = r#"
let active = null;
const $ = id => document.getElementById(id);
const stamp = () => new Date().toISOString().slice(11, 19) + ' UTC';
function log(message) {
  const area = $('studio-log');
  area.value += `[${stamp()}] ${message}\n`;
  area.scrollTop = area.scrollHeight;
}
function setProgress(percent, message) {
  $('progress-bar').style.width = `${Math.max(0, Math.min(100, percent))}%`;
  $('progress-percent').textContent = `${Math.round(percent)}%`;
  $('progress-status').textContent = message;
}
function waitMetadata(media) {
  return new Promise((resolve, reject) => {
    const done = () => { cleanup(); resolve(); };
    const fail = () => { cleanup(); reject(new Error('The browser could not read the selected media.')); };
    const cleanup = () => { media.removeEventListener('loadedmetadata', done); media.removeEventListener('error', fail); };
    media.addEventListener('loadedmetadata', done, { once: true });
    media.addEventListener('error', fail, { once: true });
  });
}
function pickMime() {
  const choices = [
    'video/mp4;codecs="avc1.42E01E,mp4a.40.2"',
    'video/mp4;codecs=h264,aac',
    'video/mp4',
    'video/webm;codecs=vp9,opus',
    'video/webm;codecs=vp8,opus',
    'video/webm'
  ];
  return choices.find(type => MediaRecorder.isTypeSupported(type)) || '';
}
function drawContained(ctx, media, width, height, scale, dx, dy, blurred) {
  const sw = media.videoWidth || media.naturalWidth;
  const sh = media.videoHeight || media.naturalHeight;
  if (!sw || !sh) return;
  const cover = Math.max(width / sw, height / sh);
  ctx.save();
  ctx.filter = blurred ? 'blur(34px) brightness(0.52) saturate(1.35)' : 'none';
  const factor = blurred ? cover * 1.12 : Math.min(width / sw, height / sh) * scale;
  const dw = sw * factor, dh = sh * factor;
  ctx.drawImage(media, (width - dw) / 2 + dx, (height - dh) / 2 + dy, dw, dh);
  ctx.restore();
}
function drawFrame(ctx, visual, kind, motion, elapsed, duration) {
  const w = ctx.canvas.width, h = ctx.canvas.height;
  ctx.clearRect(0, 0, w, h);
  const p = duration ? Math.min(1, elapsed / duration) : 0;
  drawContained(ctx, visual, w, h, 1, 0, 0, true);
  ctx.fillStyle = 'rgba(2,7,12,.18)'; ctx.fillRect(0, 0, w, h);
  let scale = 0.94, dx = 0, dy = 0;
  if (kind === 'image') {
    if (motion === 'in') scale = 0.94 + p * 0.06;
    else if (motion === 'out') scale = 1.0 - p * 0.06;
    else if (motion === 'drift') { scale = 0.96; dx = (p - .5) * 34; dy = Math.sin(p * Math.PI * 2) * 9; }
    else { scale = 0.94 + p * 0.035; dx = Math.sin(p * Math.PI) * 18 - 9; dy = Math.cos(p * Math.PI) * 8; }
  }
  drawContained(ctx, visual, w, h, scale, dx, dy, false);
}
function downloadBlob(url, name) {
  const a = document.createElement('a'); a.href = url; a.download = name; a.click();
  log(`Download requested: ${name}`);
}
export function cancelStudioRender() {
  if (!active) return;
  active.cancelled = true;
  active.audio?.pause(); active.video?.pause();
  if (active.recorder?.state === 'recording') active.recorder.stop();
  log('CANCELLED: Render stopped by user.');
  setProgress(0, 'Render cancelled.');
}
export async function startStudioRender() {
  if (active) throw new Error('A render is already running.');
  const audioFile = $('audio-file').files[0];
  const visualFile = $('visual-file').files[0];
  if (!audioFile) throw new Error('Choose an MP3 or WAV soundtrack.');
  if (!visualFile) throw new Error('Choose a JPG, PNG, WebP, or MP4 visual.');
  if (!window.MediaRecorder || !$('studio-canvas').captureStream) throw new Error('This browser cannot record a canvas video.');

  const mime = pickMime();
  if (!mime) throw new Error('This browser exposes no compatible MP4 or WebM recorder.');
  const isMp4 = mime.startsWith('video/mp4');
  const state = { cancelled: false, audio: null, video: null, recorder: null };
  active = state;
  $('render-button').disabled = true;
  $('cancel-button').hidden = false;
  $('result-wrap').hidden = true;
  $('studio-log').value = 'Fade Forge Studio processing log\n';
  setProgress(0, 'Preparing local media…');
  log(`Soundtrack: ${audioFile.name} (${(audioFile.size / 1048576).toFixed(2)} MB)`);
  log(`Visual: ${visualFile.name} (${(visualFile.size / 1048576).toFixed(2)} MB)`);
  log(`Recorder selected: ${mime || 'browser default'}`);
  if (!isMp4) log('NOTICE: MP4 recording is unavailable here; this browser will produce WebM.');

  let audioUrl, visualUrl, context;
  try {
    audioUrl = URL.createObjectURL(audioFile); visualUrl = URL.createObjectURL(visualFile);
    const audio = new Audio(audioUrl); state.audio = audio; audio.preload = 'auto';
    await waitMetadata(audio);
    if (!Number.isFinite(audio.duration) || audio.duration <= 0) throw new Error('The soundtrack duration is invalid.');
    const kind = visualFile.type.startsWith('video/') || visualFile.name.toLowerCase().endsWith('.mp4') ? 'video' : 'image';
    let visual;
    if (kind === 'video') {
      visual = document.createElement('video'); state.video = visual;
      visual.src = visualUrl; visual.muted = true; visual.loop = true; visual.playsInline = true; visual.preload = 'auto';
      await waitMetadata(visual);
      log(`Source MP4: ${visual.duration.toFixed(2)}s; original audio excluded; looping enabled.`);
    } else {
      visual = new Image(); visual.src = visualUrl; await visual.decode();
      log(`Image decoded: ${visual.naturalWidth} × ${visual.naturalHeight}; edge-safe motion enabled.`);
    }

    const canvas = $('studio-canvas');
    const size = Number($('output-size').value); canvas.width = size; canvas.height = size;
    const ctx = canvas.getContext('2d', { alpha: false });
    drawFrame(ctx, visual, kind, $('motion-style').value, 0, audio.duration);
    const canvasStream = canvas.captureStream(30);
    context = new AudioContext();
    const source = context.createMediaElementSource(audio);
    const destination = context.createMediaStreamDestination();
    source.connect(destination);
    const stream = new MediaStream([...canvasStream.getVideoTracks(), ...destination.stream.getAudioTracks()]);
    const chunks = [];
    const recorder = new MediaRecorder(stream, { mimeType: mime, videoBitsPerSecond: 14000000, audioBitsPerSecond: 192000 });
    state.recorder = recorder;
    recorder.ondataavailable = event => { if (event.data.size) chunks.push(event.data); };
    const stopped = new Promise(resolve => recorder.addEventListener('stop', resolve, { once: true }));
    recorder.start(1000);
    if (kind === 'video') await visual.play();
    await context.resume();
    await audio.play();
    log(`RENDER: ${size} × ${size}, 30 FPS, 14 Mbps target, soundtrack ${audio.duration.toFixed(2)}s.`);
    const started = performance.now();
    await new Promise(resolve => {
      const frame = () => {
        if (state.cancelled || audio.ended || audio.currentTime >= audio.duration - .025) { resolve(); return; }
        if (kind === 'video' && visual.readyState >= 2 || kind === 'image') drawFrame(ctx, visual, kind, $('motion-style').value, audio.currentTime, audio.duration);
        const percent = audio.currentTime / audio.duration * 100;
        const remaining = Math.max(0, audio.duration - audio.currentTime);
        setProgress(percent, `Rendering locally — ${remaining.toFixed(1)} seconds remaining`);
        requestAnimationFrame(frame);
      };
      requestAnimationFrame(frame);
    });
    audio.pause(); if (kind === 'video') visual.pause();
    if (recorder.state === 'recording') recorder.stop();
    await stopped;
    stream.getTracks().forEach(track => track.stop());
    if (state.cancelled) return;
    if (!chunks.length) throw new Error('The recorder completed without producing video data.');
    const blob = new Blob(chunks, { type: recorder.mimeType || mime });
    const resultUrl = URL.createObjectURL(blob);
    const ext = isMp4 ? 'mp4' : 'webm';
    const base = audioFile.name.replace(/\.[^.]+$/, '').replace(/[^a-z0-9_-]+/gi, '-').replace(/^-|-$/g, '') || 'fade-forge-studio';
    const name = `${base}-studio.${ext}`;
    const player = $('result-video'); player.src = resultUrl;
    $('result-format').textContent = `${ext.toUpperCase()} • ${(blob.size / 1048576).toFixed(1)} MB • ${audio.duration.toFixed(1)}s`;
    $('download-button').onclick = () => downloadBlob(resultUrl, name);
    $('result-wrap').hidden = false;
    setProgress(100, `Finished in ${((performance.now() - started) / 1000).toFixed(1)} seconds.`);
    log(`SUCCESS: ${name}, ${(blob.size / 1048576).toFixed(2)} MB.`);
  } catch (error) {
    log(`ERROR: ${error?.message || error}`);
    setProgress(0, error?.message || String(error));
    throw error;
  } finally {
    state.audio?.pause(); state.video?.pause();
    if (context) await context.close().catch(() => {});
    if (audioUrl) URL.revokeObjectURL(audioUrl);
    if (visualUrl) URL.revokeObjectURL(visualUrl);
    $('render-button').disabled = false;
    $('cancel-button').hidden = true;
    active = null;
  }
}
"#)]
extern "C" {
    #[wasm_bindgen(js_name = startStudioRender)]
    fn start_studio_render() -> Promise;

    #[wasm_bindgen(js_name = cancelStudioRender)]
    fn cancel_studio_render();
}

#[function_component(App)]
fn app() -> Html {
    let start = Callback::from(move |_| {
        spawn_local(async move {
            let _ = JsFuture::from(start_studio_render()).await;
        });
    });
    let cancel = Callback::from(move |_| cancel_studio_render());

    html! {
        <main class="app">
            <header class="hero">
                <div>
                    <p class="eyebrow">{"MIKEGYVER STUDIO • LOCAL MEDIA LAB"}</p>
                    <h1>{"Fade Forge Studio"}</h1>
                    <p class="subtitle">{"Turn one soundtrack and one visual into a finished, soundtrack-length video."}</p>
                </div>
                <span class="badge">{"MVPV2 • RUST • WASM"}</span>
            </header>

            <section class="panel">
                <h2>{"1. Load your media"}</h2>
                <p class="panel-intro">{"Files remain on this device. Existing MP4 audio is never included."}</p>
                <div class="upload-grid">
                    <label class="upload"><strong>{"🎵 Soundtrack"}</strong><small>{"MP3 or WAV"}</small><input id="audio-file" type="file" accept=".mp3,.wav,audio/mpeg,audio/wav" /></label>
                    <label class="upload"><strong>{"🎬 Visual"}</strong><small>{"JPG, PNG, WebP, or MP4"}</small><input id="visual-file" type="file" accept=".jpg,.jpeg,.png,.webp,.mp4,image/jpeg,image/png,image/webp,video/mp4" /></label>
                </div>
            </section>

            <section class="panel">
                <h2>{"2. Direct the render"}</h2>
                <p class="panel-intro">{"Still images keep every edge visible over a matching blurred backdrop. MP4 visuals loop automatically."}</p>
                <div class="controls">
                    <label class="field">{"Motion style"}<select id="motion-style"><option value="combo">{"Cinematic Combination"}</option><option value="in">{"Slow Zoom In"}</option><option value="out">{"Slow Zoom Out"}</option><option value="drift">{"Gentle Drift"}</option></select></label>
                    <label class="field">{"Output size"}<select id="output-size"><option value="1080">{"1080 × 1080"}</option><option value="720">{"720 × 720 (faster)"}</option></select></label>
                    <label class="field">{"Sound policy"}<span class="switch"><input type="checkbox" checked=true disabled=true />{"Replace all visual audio"}</span></label>
                </div>
                <div class="summary">
                    <div class="metric"><span>{"VIDEO"}</span><strong>{"30 FPS"}</strong></div>
                    <div class="metric"><span>{"TARGET"}</span><strong>{"14 Mbps"}</strong></div>
                    <div class="metric"><span>{"DURATION"}</span><strong>{"Soundtrack length"}</strong></div>
                    <div class="metric"><span>{"PRIVACY"}</span><strong>{"Local only"}</strong></div>
                </div>
                <div class="actions">
                    <button id="render-button" class="primary" onclick={start}>{"Forge Soundtrack Video"}</button>
                    <button id="cancel-button" class="cancel" hidden=true onclick={cancel}>{"Cancel Render"}</button>
                </div>
                <div class="progress-shell"><div id="progress-bar" class="progress-bar"></div></div>
                <p class="progress-line"><span id="progress-status">{"Ready for media."}</span><strong id="progress-percent">{"0%"}</strong></p>
                <p class="privacy">{"Rendering occurs in real time so the finished recording stays synchronized with the complete soundtrack."}</p>
            </section>

            <section class="panel">
                <h2>{"Live composition"}</h2>
                <p class="panel-intro">{"The canvas below is the exact square composition sent to the recorder."}</p>
                <div class="stage"><canvas id="studio-canvas" width="1080" height="1080"></canvas><span class="stage-note">{"EDGE-SAFE • FULL ARTWORK VISIBLE"}</span></div>
            </section>

            <section id="result-wrap" class="panel" hidden=true>
                <div class="result-head"><h2>{"✓ Finished video"}</h2><span id="result-format"></span></div>
                <video id="result-video" controls=true playsinline=true></video>
                <button id="download-button" class="download">{"Download Finished Video"}</button>
            </section>

            <section class="panel">
                <div class="log-head"><h2>{"Processing log"}</h2><span>{"LOCAL • COPYABLE"}</span></div>
                <textarea id="studio-log" readonly=true aria-label="Fade Forge Studio processing log">{"Fade Forge Studio processing log\n[ready] Choose a soundtrack and visual.\n"}</textarea>
            </section>
        </main>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
