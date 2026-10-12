//! Text-based editing commands (`transcript.*`): the Text panel ▸ Transcript tab.
//!
//! Transcripts belong to media items (`Project::transcripts`, media time); the sequence transcript
//! is derived from them ([`filmcraft_edit::transcript::sequence_words`]). Words of the sequence
//! transcript are addressed by index (`from`, `to`, inclusive), as `transcript.inspect` lists them.
//!
//! Speech recognition goes through a [`Transcriber`]: [`Session::transcriber`] when a host or a
//! test installed one, else the Whisper model named by `model` from `<data dir>/models` (needs the
//! engine feature `whisper`; without it `transcript.generate` fails with a clear error, and agents
//! can still bring their own transcript with `transcript.set`).

use std::sync::Arc;

use serde_json::{Value, json};

use filmcraft_edit as edit;
use filmcraft_edit::transcript::{self as tx, CaptionRules, SeqWord};
use filmcraft_project::{CaptionFormat, CaptionTrack, ItemId, ItemKind, TrackId, Transcript};
use filmcraft_speech::{Options, SpeechError, Transcriber};
use filmcraft_time::{TICKS_PER_SECOND, Tick, TimeRange};

use crate::commands::{CommandSpec, always, bad, bool_p, f64_p, has_seq, str_p, u64_p};
use crate::{EngineError, Result, Session};

type Run = fn(&mut Session, &Value) -> Result<Value>;
type Enabled = fn(&Session) -> std::result::Result<(), String>;

fn spec(id: &'static str, label: &'static str, menu: &'static [&'static str], params: &'static str, enabled: Enabled, run: Run, journal: bool) -> CommandSpec {
    CommandSpec { id, label, menu, shortcut: None, params, enabled, run, journal }
}

/// Where downloaded speech models live (`<data dir>/models`).
pub fn models_dir() -> Option<std::path::PathBuf> {
    crate::autosave::default_data_dir().map(|d| d.join("models"))
}

/// Whether this build can transcribe with Whisper (feature `whisper`).
pub fn speech_available() -> bool {
    filmcraft_speech::available()
}

/// The words of the active sequence's transcript.
pub fn sequence_words(s: &Session) -> Vec<SeqWord> {
    match s.active_sequence() {
        Some(q) => tx::sequence_words(q, &s.project.transcripts),
        None => Vec::new(),
    }
}

fn has_transcript(s: &Session) -> std::result::Result<(), String> {
    has_seq(s)?;
    if sequence_words(s).is_empty() { Err("the sequence has no transcript (Transcribe first)".into()) } else { Ok(()) }
}

fn has_transcripts(s: &Session) -> std::result::Result<(), String> {
    if s.project.transcripts.is_empty() { Err("there are no transcripts".into()) } else { Ok(()) }
}

/// Why speech-to-text can't run in this build (no installed transcriber, built without `whisper`).
pub(crate) const NO_SPEECH: &str =
    "speech-to-text is not available in this build (built without the `whisper` feature); import a transcript with transcript.set instead";

/// `transcript.generate` can run: a host installed a transcriber or the build has speech-to-text
/// (#97: it reported enabled and then always failed).
pub(crate) fn can_transcribe(s: &Session) -> std::result::Result<(), String> {
    if s.transcriber.is_some() || speech_available() { Ok(()) } else { Err(NO_SPEECH.into()) }
}

/// `transcript.downloadModel` can run: built with `speech-download` (#98).
fn can_download(_: &Session) -> std::result::Result<(), String> {
    if cfg!(feature = "speech-download") {
        Ok(())
    } else {
        Err("model downloads are not available in this build (built without the `speech-download` feature)".into())
    }
}

/// The media item behind a project item (subclips resolve to their parent).
fn media_item(s: &Session, item: ItemId) -> Option<ItemId> {
    s.project.resolve_media(item).map(|(root, _, _)| root)
}

fn ids_p(p: &Value, k: &str) -> Option<Vec<ItemId>> {
    p.get(k).and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_u64).map(ItemId).collect())
}

/// Items to transcribe: `items` / `item`, else the Project panel selection, else the media of the
/// active sequence's enabled audio clips. Subclips resolve to their media; duplicates are removed.
fn targets(s: &Session, p: &Value) -> Vec<ItemId> {
    let mut raw = ids_p(p, "items").or_else(|| u64_p(p, "item").map(|i| vec![ItemId(i)])).unwrap_or_default();
    if raw.is_empty() {
        raw = s.state.project_selection.clone();
    }
    if raw.is_empty()
        && let Some(q) = s.active_sequence()
    {
        raw = q.audio_tracks.iter().flat_map(|t| t.items.iter()).filter(|it| it.enabled).map(|it| it.item).collect();
    }
    let mut out = Vec::new();
    for i in raw {
        if let Some(m) = media_item(s, i)
            && !out.contains(&m)
        {
            out.push(m);
        }
    }
    out
}

/// Mono 16 kHz audio of a media item (None: no audio).
fn item_audio(s: &Session, item: ItemId) -> Option<Vec<f32>> {
    let dur = match &s.project.item(item)?.kind {
        ItemKind::Media(m) => m.duration(),
        _ => return None,
    };
    let src = s.source(item)?;
    if !src.info().has_audio() {
        return None;
    }
    let sr = filmcraft_speech::SAMPLE_RATE;
    let len = usize::try_from(dur.to_units_floor(sr as i64)).ok().filter(|n| *n > 0 && *n <= 16_000 * 60 * 60)?;
    let buf = src.audio(0, len, sr).ok()?;
    Some(filmcraft_speech::downmix(&buf.channels))
}

fn speech_err(e: SpeechError) -> EngineError {
    EngineError::Other(e.to_string())
}

/// The transcriber to use: the installed one, else the named catalogue model.
fn transcriber(s: &Session, p: &Value) -> Result<Arc<dyn Transcriber>> {
    if let Some(t) = &s.transcriber {
        return Ok(t.clone());
    }
    // Settings ▸ Media Analysis & Transcription ▸ Speech model
    let model = str_p(p, "model").unwrap_or(&s.prefs.media_analysis.whisper_model);
    if filmcraft_speech::models::find(model).is_none() {
        return Err(speech_err(SpeechError::UnknownModel(model.into())));
    }
    if !filmcraft_speech::available() {
        return Err(EngineError::Other(NO_SPEECH.into()));
    }
    let dir = models_dir().ok_or_else(|| EngineError::Other("no data directory for speech models".into()))?;
    filmcraft_speech::load(&dir, model).map_err(speech_err)
}

fn generate(s: &mut Session, p: &Value) -> Result<Value> {
    generate_progress(s, p, &mut |_, _| true)
}

fn generate_progress(s: &mut Session, p: &Value, progress: filmcraft_speech::ProgressFn<'_>) -> Result<Value> {
    let items = targets(s, p);
    if items.is_empty() {
        return Err(bad("transcript.generate", "nothing to transcribe (pass `items`, select clips, or open a sequence with audio)"));
    }
    let t = transcriber(s, p)?;
    // Settings ▸ Media Analysis & Transcription: language (or auto-detect) and speaker labelling
    let ma = &s.prefs.media_analysis;
    let default_language = if ma.language_auto_detect { None } else { Some(ma.default_language.clone()) };
    let opts = Options {
        language: match str_p(p, "language") {
            Some(l) => Some(l).filter(|l| !l.is_empty() && *l != "auto").map(str::to_string),
            None => default_language,
        },
        diarize: bool_p(p, "diarize").unwrap_or(ma.speaker_labeling != "off"),
        max_speakers: u64_p(p, "maxSpeakers").map(|n| n.clamp(1, 32) as usize).unwrap_or(Options::default().max_speakers),
    };
    let mut done: Vec<(ItemId, Transcript)> = Vec::new();
    let mut skipped = Vec::new();
    let count = items.len().max(1) as f32;
    for (index, item) in items.into_iter().enumerate() {
        let Some(audio) = item_audio(s, item) else {
            skipped.push(item.0);
            continue;
        };
        let mut tr = t.transcribe(&audio, &opts, &mut |fraction, status| progress((index as f32 + fraction) / count, status)).map_err(speech_err)?;
        tr.normalize();
        done.push((item, tr));
    }
    if done.is_empty() {
        return Err(EngineError::Other("none of the clips has audio to transcribe".into()));
    }
    let report: Vec<Value> = done
        .iter()
        .map(|(i, t)| json!({"item": i.0, "words": t.words.len(), "speakers": t.speakers.len(), "language": t.language, "source": t.source}))
        .collect();
    s.edit("Transcribe", move |pr, _| {
        for (i, t) in done {
            pr.transcripts.insert(i, Arc::new(t));
        }
        Ok(())
    })?;
    Ok(json!({"items": report, "skipped": skipped}))
}

fn set(s: &mut Session, p: &Value) -> Result<Value> {
    let item = u64_p(p, "item").map(ItemId).ok_or_else(|| bad("transcript.set", "`item` is required"))?;
    let item = media_item(s, item).ok_or_else(|| bad("transcript.set", "no such media item"))?;
    let v = p.get("transcript").cloned().ok_or_else(|| bad("transcript.set", "`transcript` is required"))?;
    let mut t: Transcript = serde_json::from_value(v).map_err(|e| bad("transcript.set", e.to_string()))?;
    if t.source.is_empty() {
        t.source = "imported".into();
    }
    t.normalize();
    t.check().map_err(|e| bad("transcript.set", e))?;
    let n = t.words.len();
    s.edit("Set Transcript", move |pr, _| {
        pr.transcripts.insert(item, Arc::new(t));
        Ok(())
    })?;
    Ok(json!({"item": item.0, "words": n}))
}

fn delete(s: &mut Session, p: &Value) -> Result<Value> {
    let items: Vec<ItemId> = match ids_p(p, "items").or_else(|| u64_p(p, "item").map(|i| vec![ItemId(i)])) {
        Some(v) => v.into_iter().filter_map(|i| media_item(s, i)).collect(),
        None => s.project.transcripts.keys().copied().collect(),
    };
    let n = items.iter().filter(|i| s.project.transcripts.contains_key(i)).count();
    if n == 0 {
        return Err(EngineError::Other("no transcript to delete".into()));
    }
    s.edit("Delete Transcript", move |pr, _| {
        for i in items {
            pr.transcripts.remove(&i);
        }
        Ok(())
    })?;
    Ok(json!({"deleted": n}))
}

fn word_json(i: usize, w: &SeqWord) -> Value {
    json!({"i": i, "text": w.text, "start": w.start.0, "end": w.end.0, "speaker": w.speaker, "clip": w.clip.0, "item": w.item.0, "confidence": w.confidence})
}

fn inspect(s: &mut Session, p: &Value) -> Result<Value> {
    let words = sequence_words(s);
    let gap = Tick::from_seconds_f64(f64_p(p, "paragraphGapSeconds").unwrap_or(1.5));
    let paras: Vec<Value> = tx::paragraphs(&words, gap)
        .into_iter()
        .map(|r| {
            let text = words[r.clone()].iter().map(|w| w.text.as_str()).collect::<Vec<_>>().join(" ");
            json!({"from": r.start, "to": r.end - 1, "speaker": words[r.start].speaker, "start": words[r.start].start.0, "end": words[r.end - 1].end.0, "text": text})
        })
        .collect();
    let speakers: Vec<String> = {
        let mut v: Vec<String> = Vec::new();
        for w in &words {
            if let Some(n) = &w.speaker
                && !v.contains(n)
            {
                v.push(n.clone());
            }
        }
        v
    };
    let current = tx::word_at(&words, s.playhead());
    Ok(json!({
        "words": words.iter().enumerate().map(|(i, w)| word_json(i, w)).collect::<Vec<_>>(),
        "paragraphs": paras,
        "speakers": speakers,
        "current": current,
        "items": s.project.transcripts.iter().map(|(i, t)| json!({"item": i.0, "words": t.words.len(), "language": t.language, "source": t.source, "speakers": t.speakers.iter().map(|k| &k.name).collect::<Vec<_>>()})).collect::<Vec<_>>(),
    }))
}

fn search(s: &mut Session, p: &Value) -> Result<Value> {
    let q = str_p(p, "query").ok_or_else(|| bad("transcript.search", "`query` is required"))?;
    let words = sequence_words(s);
    let hits: Vec<Value> = tx::search(&words, q)
        .into_iter()
        .map(|r| json!({"from": r.start, "to": r.end - 1, "start": words[r.start].start.0, "end": words[r.end - 1].end.0}))
        .collect();
    Ok(json!({"matches": hits}))
}

/// Timeline range of the words `from..=to` (frame-snapped outward).
fn range_p(s: &Session, p: &Value, cmd: &str) -> Result<TimeRange> {
    let words = sequence_words(s);
    let from = u64_p(p, "from").ok_or_else(|| bad(cmd, "`from` (word index) is required"))? as usize;
    let to = u64_p(p, "to").map(|n| n as usize).unwrap_or(from);
    tx::word_range(&words, from, to, s.sequence_rate()).ok_or_else(|| bad(cmd, format!("word index out of range (the transcript has {} words)", words.len())))
}

fn range_json(r: TimeRange) -> Value {
    json!({"start": r.start.0, "end": r.end().0})
}

fn select(s: &mut Session, p: &Value) -> Result<Value> {
    let r = range_p(s, p, "transcript.select")?;
    let fd = s.sequence_rate().frame_duration();
    s.edit_sequence("Mark Transcript Selection", |q, _, _| {
        q.mark_in = Some(r.start);
        q.mark_out = Some(r.end() - fd);
        Ok(())
    })?;
    s.set_playhead(r.start);
    Ok(range_json(r))
}

fn extract_or_lift(s: &mut Session, p: &Value, extract: bool) -> Result<Value> {
    let cmd = if extract { "transcript.extract" } else { "transcript.lift" };
    let r = range_p(s, p, cmd)?;
    let tg = s.targeting().targeted;
    s.edit_sequence(if extract { "Extract Text" } else { "Lift Text" }, |q, ctx, _| {
        if extract {
            edit::extract(q, &tg, r, ctx);
        } else {
            edit::lift(q, &tg, r, ctx);
        }
        q.mark_in = None;
        q.mark_out = None;
        Ok(())
    })?;
    s.set_playhead(r.start);
    Ok(range_json(r))
}

fn rename_speaker(s: &mut Session, p: &Value) -> Result<Value> {
    let name = str_p(p, "name").map(str::trim).filter(|n| !n.is_empty()).ok_or_else(|| bad("transcript.renameSpeaker", "`name` is required"))?.to_string();
    let item = u64_p(p, "item").map(ItemId).and_then(|i| media_item(s, i));
    // `speaker`: the current name (every transcript), or an index (needs `item`)
    let (old_name, index) = match p.get("speaker") {
        Some(Value::String(n)) => (Some(n.clone()), None),
        Some(v) if v.is_u64() => (None, v.as_u64().map(|n| n as usize)),
        _ => return Err(bad("transcript.renameSpeaker", "`speaker` (name, or index with `item`) is required")),
    };
    if index.is_some() && item.is_none() {
        return Err(bad("transcript.renameSpeaker", "a speaker index needs `item`"));
    }
    let mut n = 0;
    let mut next = s.project.transcripts.clone();
    for (i, t) in next.iter_mut() {
        if item.is_some_and(|x| x != *i) {
            continue;
        }
        let tt = Arc::make_mut(t);
        for (k, sp) in tt.speakers.iter_mut().enumerate() {
            if old_name.as_ref().is_some_and(|o| *o == sp.name) || index == Some(k) {
                sp.name = name.clone();
                n += 1;
            }
        }
    }
    if n == 0 {
        return Err(EngineError::Other("no such speaker".into()));
    }
    s.edit("Rename Speaker", move |pr, _| {
        pr.transcripts = next;
        Ok(())
    })?;
    Ok(json!({"renamed": n}))
}

fn remove_ranges(s: &mut Session, label: &str, ranges: Vec<TimeRange>) -> Result<Value> {
    let n = ranges.len();
    if n == 0 {
        return Ok(json!({"removed": 0, "ticks": 0}));
    }
    let total = s.edit_sequence(label, |q, ctx, _| Ok(tx::ripple_delete_ranges(q, ranges, ctx)))?;
    Ok(json!({"removed": n, "ticks": total.0, "seconds": total.0 as f64 / TICKS_PER_SECOND as f64}))
}

fn remove_pauses(s: &mut Session, p: &Value) -> Result<Value> {
    let words = sequence_words(s);
    let min = Tick::from_seconds_f64(f64_p(p, "minSeconds").unwrap_or(1.0));
    let keep = Tick::from_seconds_f64(f64_p(p, "keepSeconds").unwrap_or(0.15));
    let ranges = tx::find_pauses(&words, min, keep, s.sequence_rate());
    remove_ranges(s, "Remove Pauses", ranges)
}

fn remove_fillers(s: &mut Session, p: &Value) -> Result<Value> {
    let words = sequence_words(s);
    let fillers: Vec<String> = match p.get("fillers").and_then(Value::as_array) {
        Some(a) => a.iter().filter_map(Value::as_str).map(str::to_string).collect(),
        None => tx::DEFAULT_FILLERS.iter().map(|f| f.to_string()).collect(),
    };
    let hits = tx::find_fillers(&words, &fillers);
    let ranges = tx::filler_ranges(&words, &hits, s.sequence_rate());
    remove_ranges(s, "Remove Filler Words", ranges)
}

fn create_captions(s: &mut Session, p: &Value) -> Result<Value> {
    let words = sequence_words(s);
    captions_from_words(s, p, &words)
}

fn captions_from_words(s: &mut Session, p: &Value, words: &[SeqWord]) -> Result<Value> {
    let d = CaptionRules::default();
    let rules = CaptionRules {
        max_chars: u64_p(p, "maxChars").map(|n| n as usize).unwrap_or(d.max_chars),
        lines: u64_p(p, "lines").map(|n| n as usize).unwrap_or(d.lines),
        min_duration: f64_p(p, "minSeconds").map(Tick::from_seconds_f64).unwrap_or(d.min_duration),
        max_duration: f64_p(p, "maxSeconds").map(Tick::from_seconds_f64).unwrap_or(d.max_duration),
        gap_frames: p.get("gapFrames").and_then(Value::as_i64).unwrap_or(d.gap_frames),
        break_pause: d.break_pause,
    };
    let blocks = tx::caption_blocks(words, &rules, s.sequence_rate());
    let format = str_p(p, "format").and_then(CaptionFormat::from_name).unwrap_or_default();
    let name = str_p(p, "name").unwrap_or("Transcript").to_string();
    let language = str_p(p, "language").unwrap_or("en").to_string();
    let n = blocks.len();
    let tid = s.edit_sequence("Create Captions", |q, ctx, st| {
        let tid = TrackId(ctx.alloc());
        let mut t = CaptionTrack::new(tid, name, format);
        t.language = language;
        t.captions = tx::blocks_to_captions(&blocks, ctx);
        q.caption_tracks.insert(0, t);
        st.caption_selection.clear();
        Ok(tid)
    })?;
    Ok(json!({"track": tid.0, "captions": n}))
}

/// Timed graphic clips use the existing project format and normal text editing tools.
fn text_from_words(s: &mut Session, words: &[SeqWord]) -> Result<Vec<filmcraft_project::ClipId>> {
    use filmcraft_geom::Vec2;
    use filmcraft_project::{Param, ParamValue, Track, TrackKind};

    let rules = CaptionRules { max_chars: 32, max_duration: Tick::from_seconds_f64(4.0), ..Default::default() };
    let blocks = tx::caption_blocks(words, &rules, s.sequence_rate());
    let seq = s.state.active_sequence.ok_or(EngineError::NoSequence)?;
    let font = s.prefs.graphics.default_font.clone();
    let ligatures = s.prefs.graphics.ligatures;
    s.edit("Create Editable Text", |pr, _| {
        let settings = pr.sequence(seq).ok_or(EngineError::NoSequence)?.settings.clone();
        let (w, h, rate) = (settings.width, settings.height, settings.frame_rate);
        let source = crate::graphics::graphic_source(pr, w, h, rate);
        let mut track = Track::new(TrackId(pr.alloc_id()), TrackKind::Video, "Generated text".into());
        let mut clips = Vec::new();
        let size = (f64::from(w).min(f64::from(h)) * 0.05).max(1.0);
        for block in blocks {
            let mut layer = filmcraft_project::graphic::new_text_layer(&block.text, Vec2::new(f64::from(w) * 0.1, f64::from(h) * 0.78), size);
            for (id, value) in [
                ("align", ParamValue::Choice(1)),
                ("box_width", ParamValue::Float(f64::from(w) * 0.8)),
                ("stroke", ParamValue::Bool(true)),
                ("stroke_width", ParamValue::Float(size * 0.06)),
                ("ligatures", ParamValue::Bool(ligatures)),
            ] {
                layer.params.insert(id.into(), Param::new(value));
            }
            if !font.trim().is_empty() {
                layer.params.insert("font".into(), Param::new(ParamValue::Text(font.trim().into())));
            }
            let duration = block.end - block.start;
            let mut clip = pr
                .make_track_item(source, TrackKind::Video, block.start, TimeRange::new(Tick::ZERO, duration), rate)
                .ok_or_else(|| bad("graphics.autoGenerateText", "cannot create a timed text clip"))?;
            clip.name = block.text;
            clip.effects.push(layer);
            for effect in &mut clip.effects {
                filmcraft_project::resolve_auto_points(effect, (w, h), (w, h));
            }
            clips.push(clip.id);
            track.items.push(clip);
        }
        track.sort();
        let q = pr.sequence_mut(seq).ok_or(EngineError::NoSequence)?;
        // Append above all existing video tracks so the text remains visible over B-roll.
        q.video_tracks.push(track);
        q.check().map_err(EngineError::Other)?;
        Ok(clips)
    })
}

type CaptionOutput = std::result::Result<(Arc<filmcraft_project::Project>, Vec<filmcraft_project::ClipId>), String>;
type CaptionSlot = Arc<std::sync::Mutex<Option<CaptionOutput>>>;

pub struct PendingCaptions {
    job: crate::Job,
    baseline: Arc<filmcraft_project::Project>,
    output: CaptionSlot,
    started: web_time::Instant,
    graphics: bool,
}

fn auto_captions(s: &mut Session, p: &Value, graphics: bool) -> Result<Value> {
    use std::sync::atomic::Ordering;
    let cmd = if graphics { "graphics.autoGenerateText" } else { "captions.autoGenerate" };
    has_seq(s).map_err(EngineError::Other)?;
    if !s.caption_jobs.is_empty() {
        return Err(bad(cmd, "caption/text generation is already running"));
    }
    let language = str_p(p, "language").unwrap_or("auto");
    if !["auto", "ro", "en"].contains(&language) {
        return Err(bad(cmd, "language must be auto, ro or en"));
    }
    let q = s.active_sequence().ok_or(EngineError::NoSequence)?;
    let solo = q.audio_tracks.iter().any(|t| t.enabled && t.solo && !t.muted);
    let mut audible = q.clone();
    audible.audio_tracks.retain(|t| t.enabled && !t.muted && (!solo || t.solo));
    let selected: Vec<_> = q.video_tracks.iter().chain(q.audio_tracks.iter()).flat_map(|t| &t.items).filter(|c| s.state.selection.contains(&c.id)).collect();
    let marks = q.mark_in.zip(q.mark_out).filter(|(a, b)| a < b);
    for track in &mut audible.audio_tracks {
        track.items.retain(|c| {
            c.enabled
                && c.speed.is_finite()
                && c.speed > 0.0
                && !c.reverse
                && c.frame_hold.is_none()
                && (selected.is_empty() || selected.iter().any(|v| v.id == c.id || (v.link.is_some() && v.link == c.link)))
        });
        if selected.is_empty()
            && let Some((a, b)) = marks
        {
            track.items.retain(|c| c.start < b && c.end() > a);
            for c in &mut track.items {
                let start = c.start.max(a);
                let end = c.end().min(b);
                c.source_in += Tick::from_seconds_f64((start - c.start).seconds() * c.speed);
                c.start = start;
                c.duration = end - start;
            }
        }
    }
    if !audible.audio_tracks.iter().any(|t| !t.items.is_empty()) {
        return Err(bad(cmd, "no audible audio in the selection; select its audio clip or clear the selection to caption the sequence"));
    }
    let model = str_p(p, "model").unwrap_or(&s.prefs.media_analysis.whisper_model);
    if s.transcriber.is_none() {
        let info = filmcraft_speech::models::find(model).ok_or_else(|| speech_err(SpeechError::UnknownModel(model.into())))?;
        if !models_dir().is_some_and(|d| filmcraft_speech::models::installed(&d, info)) {
            return Err(bad(cmd, "speech model is not installed; download it with transcript.downloadModel"));
        }
    }
    let mut params = json!({"language":language,"model":model,"diarize":false,"name":"Auto captions"});
    let id = s.jobs.iter().map(|j| j.id).max().unwrap_or(0).checked_add(1).ok_or_else(|| bad(cmd, "job ids exhausted"))?;
    let job = crate::Job {
        id,
        label: if graphics { "Generating editable text" } else { "Generating captions" }.into(),
        progress: Default::default(),
        result: Default::default(),
    };
    job.progress.total.store(1000, Ordering::Relaxed);
    *job.progress.status.lock().unwrap_or_else(|e| e.into_inner()) = "Loading speech model…".into();
    let mut worker = Session::new(s.services.clone());
    worker.project = s.project.clone();
    worker.media = s.media.clone();
    worker.state = s.state.clone();
    worker.prefs = s.prefs.clone();
    worker.transcriber = s.transcriber.clone();
    let output = Arc::new(std::sync::Mutex::new(None));
    let target = output.clone();
    let progress = job.progress.clone();
    let run = move || {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> CaptionOutput {
            let (words, detected) = caption_audio(&mut worker, &params, &audible, &mut |fraction, status| {
                progress.done.store((fraction.clamp(0.0, 1.0) * 1000.0) as u64, Ordering::Relaxed);
                *progress.status.lock().unwrap_or_else(|e| e.into_inner()) = status.into();
                !progress.cancel.load(Ordering::Relaxed)
            })
            .map_err(|e| e.to_string())?;
            if words.is_empty() {
                return Err("No speech was recognized in the selected audio".into());
            }
            if params["language"] == "auto" {
                params["language"] = json!(detected);
            }
            let clips = if graphics {
                text_from_words(&mut worker, &words).map_err(|e| e.to_string())?
            } else {
                captions_from_words(&mut worker, &params, &words).map_err(|e| e.to_string())?;
                Vec::new()
            };
            Ok((worker.project, clips))
        }))
        .unwrap_or_else(|_| Err("Caption generation stopped unexpectedly; the project was preserved".into()));
        *target.lock().unwrap_or_else(|e| e.into_inner()) = Some(result);
    };
    std::thread::Builder::new().name("filmcraft-auto-captions".into()).spawn(run).map_err(|e| EngineError::Other(e.to_string()))?;
    s.caption_jobs.push(PendingCaptions { job: job.clone(), baseline: s.project.clone(), output, started: web_time::Instant::now(), graphics });
    s.jobs.push(job);
    Ok(json!({"job":id,"language":language}))
}

/// Decode only the chosen timeline clips, keeping word times in each clip's source space.
fn caption_audio(
    s: &mut Session,
    p: &Value,
    audible: &filmcraft_project::Sequence,
    progress: filmcraft_speech::ProgressFn<'_>,
) -> Result<(Vec<SeqWord>, String)> {
    let t = transcriber(s, p)?;
    let language = str_p(p, "language").filter(|l| *l != "auto").map(str::to_string);
    let opts = Options { language, diarize: false, ..Default::default() };
    let clips: Vec<_> = audible.audio_tracks.iter().flat_map(|t| &t.items).collect();
    let count = clips.len().max(1) as f32;
    let mut transcripts = s.project.transcripts.clone();
    let mut detected = "en".to_string();
    for (index, clip) in clips.iter().enumerate() {
        if !progress(index as f32 / count, "Decoding selected audio…") {
            return Err(EngineError::Other("Caption generation cancelled".into()));
        }
        let source = s.source(clip.item).ok_or_else(|| EngineError::Other(format!("Cannot open audio for {}", clip.name)))?;
        if !source.info().has_audio() {
            return Err(EngineError::Other(format!("{} has no decodable audio", clip.name)));
        }
        let sr = filmcraft_speech::SAMPLE_RATE;
        let duration = Tick::from_seconds_f64(clip.duration.seconds() * clip.speed);
        let len = usize::try_from(duration.to_units_floor(i64::from(sr)))
            .ok()
            .filter(|n| *n > 0 && *n <= 16_000 * 60 * 60)
            .ok_or_else(|| bad("captions.autoGenerate", "selected audio must be between one sample and one hour"))?;
        let offset = clip.source_in.to_units_floor(i64::from(sr)).max(0);
        let buffer = source.audio(offset, len, sr).map_err(|e| EngineError::Other(format!("Cannot decode {}: {e}", clip.name)))?;
        let audio = filmcraft_speech::downmix(&buffer.channels);
        let mut tr = t.transcribe(&audio, &opts, &mut |fraction, status| progress((index as f32 + fraction) / count, status)).map_err(speech_err)?;
        if index == 0 {
            detected = tr.language.clone();
        }
        for w in &mut tr.words {
            w.start += clip.source_in;
            w.end += clip.source_in;
        }
        let mut combined = transcripts.get(&clip.item).map(|t| (**t).clone()).unwrap_or_default();
        let end = clip.source_in + duration;
        combined.words.retain(|w| w.end <= clip.source_in || w.start >= end);
        combined.language = tr.language;
        combined.source = tr.source;
        combined.words.extend(tr.words);
        combined.normalize();
        transcripts.insert(clip.item, Arc::new(combined));
    }
    let words = tx::sequence_words(audible, &transcripts);
    s.edit("Transcribe selected audio", |project, _| {
        project.transcripts = transcripts;
        Ok(())
    })?;
    Ok((words, detected))
}

pub fn poll_captions(s: &mut Session) {
    use std::sync::atomic::Ordering;
    let mut i = 0;
    while i < s.caption_jobs.len() {
        let result = s.caption_jobs[i].output.lock().unwrap_or_else(|e| e.into_inner()).take();
        let Some(result) = result else {
            i += 1;
            continue;
        };
        let pending = s.caption_jobs.remove(i);
        let outcome = if pending.job.progress.cancel.load(Ordering::Relaxed) {
            Err("Caption generation cancelled".into())
        } else if !Arc::ptr_eq(&s.project, &pending.baseline) {
            Err("The project changed during caption generation; run Generate again".into())
        } else {
            result
        };
        let outcome = outcome.and_then(|(project, clips)| {
            let playhead = s.playhead();
            let selected = s.state.active_sequence.and_then(|id| project.sequence(id)).and_then(|q| {
                clips
                    .iter()
                    .find(|id| q.find_item(**id).is_some_and(|(_, clip)| clip.range().contains(playhead)))
                    .or_else(|| clips.iter().find(|id| q.find_item(**id).is_some()))
                    .copied()
            });
            s.edit(if pending.graphics { "Generate Editable Text" } else { "Generate Auto Captions" }, |p, st| {
                *p = (*project).clone();
                if let Some(clip) = selected {
                    st.selection = vec![clip];
                    st.graphic_layers = vec![0];
                    st.caption_selection.clear();
                }
                Ok(())
            })
            .map_err(|e| e.to_string())?;
            s.toast(if pending.graphics { "Editable text generated — select a text clip to style or move it" } else { "Auto captions generated" });
            Ok(filmcraft_export::Report {
                path: if pending.graphics { "Editable text" } else { "Captions" }.into(),
                frames: 0,
                seconds: pending.started.elapsed().as_secs_f64(),
                bytes: 0,
                render_fps: 0.0,
                extra_files: Vec::new(),
            })
        });
        if let Err(e) = &outcome {
            *pending.job.progress.error.lock().unwrap_or_else(|e| e.into_inner()) = Some(e.clone());
            s.error_toast(if pending.graphics { "graphics.autoGenerateText" } else { "captions.autoGenerate" }, e.clone());
        }
        *pending.job.result.lock().unwrap_or_else(|e| e.into_inner()) = Some(outcome);
        pending.job.progress.done.store(1000, Ordering::Relaxed);
        pending.job.progress.finished.store(true, Ordering::Release);
    }
}

fn models(_: &mut Session, _: &Value) -> Result<Value> {
    let dir = models_dir();
    Ok(json!({
        "available": filmcraft_speech::available(),
        "default": filmcraft_speech::models::DEFAULT_MODEL,
        "dir": dir.as_ref().map(|d| d.to_string_lossy().to_string()),
        "models": filmcraft_speech::models::catalogue().iter().map(|m| json!({
            "id": m.id, "name": m.name, "multilingual": m.multilingual, "description": m.description,
            "license": m.license, "source": m.source, "size": m.size(),
            "installed": dir.as_ref().is_some_and(|d| filmcraft_speech::models::installed(d, m)),
        })).collect::<Vec<_>>(),
    }))
}

/// Download a catalogue model into `<data dir>/models` (feature `speech-download`). Hosts show
/// the size, source and licence (`transcript.models`) and ask before running this.
fn download_model(_: &mut Session, p: &Value) -> Result<Value> {
    let id = str_p(p, "model").unwrap_or(filmcraft_speech::models::DEFAULT_MODEL);
    let m = filmcraft_speech::models::find(id).ok_or_else(|| speech_err(SpeechError::UnknownModel(id.into())))?;
    let dir = models_dir().ok_or_else(|| EngineError::Other("no data directory for speech models".into()))?;
    #[cfg(feature = "speech-download")]
    {
        filmcraft_speech::models::download(&dir, m, &mut |_, _, _| true).map_err(speech_err)?;
        Ok(json!({"model": m.id, "dir": filmcraft_speech::models::model_dir(&dir, m).to_string_lossy()}))
    }
    #[cfg(not(feature = "speech-download"))]
    {
        let _ = (m, dir);
        Err(EngineError::Other("model downloads are not available in this build (built without the `speech-download` feature)".into()))
    }
}

pub fn commands() -> Vec<CommandSpec> {
    vec![
        spec(
            "captions.autoGenerate",
            "Generate Auto Captions…",
            &["Sequence", "Transcript"],
            r#"{"language":"auto|ro|en","model":"whisper-base"?}"#,
            can_transcribe,
            |s, p| auto_captions(s, p, false),
            true,
        ),
        spec(
            "graphics.autoGenerateText",
            "Generate Editable Text…",
            &["Graphics and Titles"],
            r#"{"language":"auto|ro|en","model":"whisper-base"?}"#,
            can_transcribe,
            |s, p| auto_captions(s, p, true),
            true,
        ),
        spec(
            "transcript.generate",
            "Transcribe…",
            &["Sequence", "Transcript"],
            r#"{"items":[id]?,"model":"whisper-base"?,"language":"en|auto"?,"diarize":bool?,"maxSpeakers":n?}"#,
            can_transcribe,
            generate,
            true,
        ),
        spec(
            "transcript.set",
            "Import Transcript",
            &[],
            r#"{"item":id,"transcript":{"language":str,"speakers":[{"name":str}],"words":[{"text":str,"start":tick,"end":tick,"speaker":n?}]}}"#,
            always,
            set,
            true,
        ),
        spec("transcript.delete", "Delete Transcript", &["Sequence", "Transcript"], r#"{"items":[id]?}"#, has_transcripts, delete, true),
        spec("transcript.inspect", "Inspect Transcript", &[], r#"{"paragraphGapSeconds":f?}"#, always, inspect, false),
        spec("transcript.search", "Search Transcript", &[], r#"{"query":str}"#, always, search, false),
        spec("transcript.models", "List Speech Models", &[], "{}", always, models, false),
        spec("transcript.downloadModel", "Download Speech Model", &[], r#"{"model":"whisper-base"?}"#, can_download, download_model, true),
        spec("transcript.select", "Mark Selected Text", &[], r#"{"from":word,"to":word?}"#, has_transcript, select, true),
        spec("transcript.extract", "Extract Selected Text", &[], r#"{"from":word,"to":word?}"#, has_transcript, |s, p| extract_or_lift(s, p, true), true),
        spec("transcript.lift", "Lift Selected Text", &[], r#"{"from":word,"to":word?}"#, has_transcript, |s, p| extract_or_lift(s, p, false), true),
        spec(
            "transcript.renameSpeaker",
            "Rename Speaker…",
            &[],
            r#"{"speaker":"Speaker 1"|index,"name":str,"item":id?}"#,
            has_transcripts,
            rename_speaker,
            true,
        ),
        spec(
            "transcript.removePauses",
            "Remove Pauses",
            &["Sequence", "Transcript"],
            r#"{"minSeconds":f?,"keepSeconds":f?}"#,
            has_transcript,
            remove_pauses,
            true,
        ),
        spec("transcript.removeFillers", "Remove Filler Words", &["Sequence", "Transcript"], r#"{"fillers":[str]?}"#, has_transcript, remove_fillers, true),
        spec(
            "transcript.createCaptions",
            "Create Captions from Transcript…",
            &["Sequence", "Transcript"],
            r#"{"maxChars":n?,"lines":1|2?,"minSeconds":f?,"maxSeconds":f?,"gapFrames":n?,"format":str?,"name":str?}"#,
            has_transcript,
            create_captions,
            true,
        ),
    ]
}
