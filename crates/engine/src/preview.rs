//! Atomic Program-monitor duplication and synchronized text positioning.

use filmcraft_geom::Vec2;
use filmcraft_project::graphic::{LayerContent, eval_layer, layer_indices};
use filmcraft_project::{ClipId, ItemKind, Param, ParamValue, Track, TrackId, TrackKind};
use filmcraft_time::Tick;
use serde_json::{Value, json};

use crate::{EngineError, Result, Session, commands};

pub fn move_clip(s: &mut Session, p: &Value) -> Result<Value> {
    let bad = |why| commands::bad("preview.move", why);
    let clip = commands::clip_p(p, "clip").ok_or_else(|| bad("need clip"))?;
    let pos = p.get("position").and_then(Value::as_array).filter(|a| a.len() == 2).ok_or_else(|| bad("need position [x,y]"))?;
    let coordinate = |n| pos.get(n).and_then(Value::as_f64).filter(|v| v.is_finite() && v.abs() <= 1e9);
    let (Some(x), Some(y)) = (coordinate(0), coordinate(1)) else { return Err(bad("invalid position")) };
    let position = Vec2::new(x, y);
    let time = commands::time_p(s, p, "");
    if time.is_none() && ["time", "frame", "seconds", "timecode"].iter().any(|k| p.get(*k).is_some()) {
        return Err(bad("invalid time"));
    }
    let time = time.unwrap_or(s.playhead());
    let duplicate = p.get("duplicate").and_then(Value::as_bool).unwrap_or(false);
    let layer = match p.get("layer") {
        Some(v) => Some(v.as_u64().and_then(|v| usize::try_from(v).ok()).ok_or_else(|| bad("invalid layer"))?),
        None => None,
    };
    let q = s.active_sequence().ok_or(EngineError::NoSequence)?;
    let (track, original) = q.find_item(clip).ok_or_else(|| bad("no such clip"))?;
    let index = q.video_tracks.iter().position(|t| t.id == track && !t.locked && t.enabled).ok_or_else(|| bad("need an unlocked video track"))?;
    let size = s.project.source_size(original.item).ok_or_else(|| bad("no video source"))?;
    if !original.enabled || !original.effect("motion").is_some_and(|e| e.enabled) {
        return Err(bad("clip or Motion is disabled"));
    }
    let graphic = matches!(s.project.item(original.item).map(|i| &i.kind), Some(ItemKind::Graphic { .. }));
    let indices = layer_indices(&original.effects);
    let ei = if graphic {
        Some(*layer.and_then(|n| indices.get(n)).ok_or_else(|| bad("need a valid graphic layer"))?)
    } else {
        if layer.is_some() || matches!(s.project.item(original.item).map(|i| &i.kind), Some(ItemKind::AdjustmentLayer { .. })) {
            return Err(bad("unsupported preview element"));
        }
        None
    };
    let mut targets = Vec::new();
    if !duplicate && ei.is_some() {
        let mt = original.source_time_at(time.clamp(original.start, (original.end() - Tick(1)).max(original.start)));
        let frame_position = filmcraft_render::motion_matrix(q, original, size, None, mt).apply(position);
        if let Some(v) = p.get("texts") {
            let ids = v.as_array().filter(|a| a.len() <= 1024).ok_or_else(|| bad("texts must be at most 1024 clip ids"))?;
            for id in ids {
                let id = ClipId(id.as_u64().ok_or_else(|| bad("invalid text clip id"))?);
                if id == clip || targets.iter().any(|(c, _, _)| *c == id) {
                    continue;
                }
                let (tr, it) = q.find_item(id).ok_or_else(|| bad("no such selected clip"))?;
                if q.track(tr).is_none_or(|t| t.locked || !t.enabled) {
                    continue;
                }
                let Some(ItemKind::Graphic { width, height, .. }) = s.project.item(it.item).map(|i| &i.kind) else { continue };
                let mt = it.source_time_at(time.clamp(it.start, (it.end() - Tick(1)).max(it.start)));
                let matrix = filmcraft_render::motion_matrix(q, it, (*width, *height), None, mt);
                let local = matrix.inverse().ok_or_else(|| bad("selected graphic has a singular transform"))?.apply(frame_position);
                if !local.x.is_finite() || !local.y.is_finite() || local.x.abs() > 1e9 || local.y.abs() > 1e9 {
                    return Err(bad("selected graphic position is out of range"));
                }
                let layers: Vec<usize> = layer_indices(&it.effects)
                    .into_iter()
                    .filter(|&n| {
                        it.effects.get(n).and_then(|e| eval_layer(e, mt, (*width, *height))).is_some_and(|sp| matches!(sp.content, LayerContent::Text(_)))
                    })
                    .collect();
                targets.push((id, layers, local));
            }
        }
    }
    if duplicate && (q.video_tracks.len() >= crate::sequence_tools::MAX_TRACKS || s.project.next_id.checked_add(2).is_none()) {
        return Err(bad("cannot allocate another track"));
    }
    let mut original = original.clone();
    let gesture = p.get("gesture").map(|v| v.as_u64().ok_or_else(|| bad("invalid gesture"))).transpose()?.unwrap_or(clip.0);
    let merge = p.get("merge").and_then(Value::as_bool).unwrap_or(false).then(|| format!("preview.move:{gesture}"));
    if p.get("begin").and_then(Value::as_bool).unwrap_or(false) {
        s.history.merge_key = None;
    }
    // The label must stay identical across the initial duplication and subsequent moves,
    // because history coalescing checks both the gesture key and the label.
    let label = "Move or Duplicate Preview Element";
    let result = s.edit_sequence_as(label, merge.as_deref(), |q, ctx, state| {
        let (id, effect_index) = if duplicate {
            original.id = ClipId(ctx.alloc());
            original.link = None;
            original.group = None;
            // A graphic clip can contain several layers: duplicate only the grabbed one,
            // while retaining intrinsic Motion/Opacity and the clip's other effects.
            if let Some(ei) = ei {
                original.effects = original.effects.into_iter().enumerate().filter(|(n, _)| !indices.contains(n) || *n == ei).map(|(_, e)| e).collect();
            }
            let effect_index = ei.and_then(|_| layer_indices(&original.effects).first().copied());
            let id = original.id;
            let mut tr = Track::new(TrackId(ctx.alloc()), TrackKind::Video, format!("{} copy", original.name));
            tr.items.push(original);
            q.video_tracks.insert(index + 1, tr);
            state.selection = vec![id];
            state.graphic_layers = if graphic { vec![0] } else { Vec::new() };
            (id, effect_index)
        } else {
            (clip, ei)
        };
        set_position(q, id, effect_index, time, position)?;
        for (id, layers, pos) in &targets {
            for ei in layers {
                set_position(q, *id, Some(*ei), time, *pos)?;
            }
        }
        Ok(id)
    })?;
    Ok(json!({"clip": result.0, "layer": if duplicate { 0 } else { layer.unwrap_or(0) }}))
}

fn set_position(q: &mut filmcraft_project::Sequence, clip: ClipId, layer: Option<usize>, time: Tick, position: Vec2) -> Result<()> {
    let (_, it) = q.find_item_mut(clip).ok_or(EngineError::NoSequence)?;
    let mt = it.source_time_at(time.clamp(it.start, (it.end() - Tick(1)).max(it.start)));
    let effect = match layer {
        Some(n) => it.effects.get_mut(n),
        None => it.effects.iter_mut().find(|e| e.effect == "motion"),
    }
    .ok_or_else(|| commands::bad("preview.move", "no position effect"))?;
    effect.params.entry("position".into()).or_insert_with(|| Param::new(ParamValue::Vec2(position))).set_at(mt, ParamValue::Vec2(position));
    Ok(())
}

/// Prepare every sequence in a desktop project, preserving existing tracks and media.
pub fn ensure_tracks(s: &mut Session, _: &Value) -> Result<Value> {
    let ids: Vec<_> =
        s.project.sequences().filter(|i| i.as_sequence().is_some_and(|q| q.video_tracks.len() < 10 || q.audio_tracks.len() < 10)).map(|i| i.id).collect();
    if ids.is_empty() {
        return Ok(Value::Null);
    }
    s.edit("Prepare Ten Video and Audio Tracks", |p, _| {
        for id in ids {
            for (kind, name) in [(TrackKind::Video, "Video"), (TrackKind::Audio, "Audio")] {
                let have = p.sequence(id).ok_or(EngineError::NoSequence)?.tracks(kind).len();
                for n in have..10 {
                    if p.next_id.checked_add(1).is_none() {
                        return Err(commands::bad("preview.ensureTracks", "track ids exhausted"));
                    }
                    let track = Track::new(TrackId(p.alloc_id()), kind, format!("{name} {}", n + 1));
                    p.sequence_mut(id).ok_or(EngineError::NoSequence)?.tracks_mut(kind).push(track);
                }
            }
        }
        Ok(())
    })?;
    Ok(Value::Null)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn demo() -> Session {
        let mut s = Session::default();
        s.execute("file.openDemoProject", json!({})).unwrap();
        s
    }

    #[test]
    fn duplicate_retains_timing_style_and_original_and_merges_the_entire_gesture() {
        let mut s = demo();
        let text =
            s.execute("graphics.newText", json!({"text":"Copy me","position":[300,250],"size":87,"seconds":3,"time":0})).unwrap()["clip"].as_u64().unwrap();
        s.execute("graphics.newText", json!({"clip":text,"text":"Other internal layer"})).unwrap();
        let before = s.project.clone();
        let q = s.active_sequence().unwrap();
        let (track, original) = q.find_item(ClipId(text)).unwrap();
        let original = original.clone();
        let index = q.video_tracks.iter().position(|t| t.id == track).unwrap();
        let n = s.history.undo.len();
        let copy = s
            .execute("preview.move", json!({"clip":text,"layer":0,"position":[600,350],"duplicate":true,"merge":true,"begin":true,"gesture":text}))
            .unwrap()["clip"]
            .as_u64()
            .unwrap();
        s.execute("preview.move", json!({"clip":copy,"layer":0,"position":[650,400],"merge":true,"gesture":text})).unwrap();
        let q = s.active_sequence().unwrap();
        assert_eq!(q.find_item(ClipId(text)).unwrap().1, &original);
        let (track, it) = q.find_item(ClipId(copy)).unwrap();
        assert_eq!(q.video_tracks[index + 1].id, track);
        assert_eq!((it.start, it.duration, it.source_in, it.speed), (original.start, original.duration, original.source_in, original.speed));
        let layers = layer_indices(&it.effects);
        assert_eq!(layers.len(), 1);
        assert_eq!(it.effects[layers[0]].param("size"), original.effects[layer_indices(&original.effects)[0]].param("size"));
        assert_eq!(s.history.undo.len(), n + 1);
        let after = s.project.clone();
        s.execute("edit.undo", json!({})).unwrap();
        assert_eq!(s.project, before);
        s.execute("edit.redo", json!({})).unwrap();
        assert_eq!(s.project, after);

        let movie = s.active_sequence().unwrap().video_tracks[0].items[0].id;
        let original = s.active_sequence().unwrap().find_item(movie).unwrap().1.clone();
        let copy = s.execute("preview.move", json!({"clip":movie.0,"position":[850,700],"duplicate":true})).unwrap()["clip"].as_u64().unwrap();
        let copied = s.active_sequence().unwrap().find_item(ClipId(copy)).unwrap().1;
        assert_eq!((copied.item, copied.start, copied.duration, copied.source_in), (original.item, original.start, original.duration, original.source_in));
        assert_eq!(copied.effect("motion").unwrap().param("scale"), original.effect("motion").unwrap().param("scale"));
        assert_eq!(copied.link, None);
        assert_eq!(s.active_sequence().unwrap().find_item(movie).unwrap().1, &original);
    }

    #[test]
    fn nonconcurrent_selected_texts_share_position_and_bad_input_is_atomic() {
        let mut s = demo();
        let a =
            s.execute("graphics.newText", json!({"newClip":true,"text":"First","position":[300,300],"time":0,"seconds":2})).unwrap()["clip"].as_u64().unwrap();
        let b = s
            .execute("graphics.newText", json!({"newClip":true,"text":"Later","position":[700,700],"time":Tick::from_seconds_f64(5.0).0,"seconds":2}))
            .unwrap()["clip"]
            .as_u64()
            .unwrap();
        s.execute("preview.move", json!({"clip":a,"layer":0,"position":[900,400],"texts":[a,b],"time":0})).unwrap();
        for c in [a, b] {
            let layers = s.execute("graphics.list", json!({"clip":c})).unwrap();
            assert_eq!(layers["layers"][0]["position"], json!([900.0, 400.0]));
        }
        for p in [
            json!({"clip":a,"layer":0,"position":[1e30,0]}),
            json!({"clip":a,"layer":0,"position":[1,2],"texts":[u64::MAX]}),
            json!({"clip":a,"layer":u64::MAX,"position":[1,2],"duplicate":true}),
            json!({"clip":a,"layer":0,"position":[1,2],"time":"invalid"}),
        ] {
            let before = s.project.clone();
            let history = s.history.undo.len();
            assert!(s.execute("preview.move", p).is_err());
            assert_eq!(s.project, before);
            assert_eq!(s.history.undo.len(), history);
        }
        let track = s.active_sequence().unwrap().find_item(ClipId(a)).unwrap().0;
        s.execute("timeline.setTrack", json!({"track":track.0,"locked":true})).unwrap();
        let before = s.project.clone();
        assert!(s.execute("preview.move", json!({"clip":a,"layer":0,"position":[1,2],"duplicate":true})).is_err());
        assert_eq!(s.project, before);
    }

    #[test]
    fn minimum_tracks_preserve_existing_tracks_and_do_not_grow_on_repeat() {
        let mut s = demo();
        let original = s.active_sequence().unwrap().clone();
        s.execute("preview.ensureTracks", json!({})).unwrap();
        let q = s.active_sequence().unwrap();
        assert_eq!((q.video_tracks.len(), q.audio_tracks.len()), (10, 10));
        assert_eq!(&q.video_tracks[..original.video_tracks.len()], &original.video_tracks);
        assert_eq!(&q.audio_tracks[..original.audio_tracks.len()], &original.audio_tracks);
        let before = s.project.clone();
        let history = s.history.undo.len();
        s.execute("preview.ensureTracks", json!({})).unwrap();
        assert_eq!(s.project, before);
        assert_eq!(s.history.undo.len(), history);
        s.execute("file.newSequence", json!({"video":12,"audio":14})).unwrap();
        s.execute("preview.ensureTracks", json!({})).unwrap();
        let q = s.active_sequence().unwrap();
        assert_eq!((q.video_tracks.len(), q.audio_tracks.len()), (12, 14));
    }
}
