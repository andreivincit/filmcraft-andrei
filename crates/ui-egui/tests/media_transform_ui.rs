//! Acceptance: click a video/still above the base clip, drag its body and corners,
//! inspect Motion and undo/redo through the same control channel used by agents.

use egui_kittest::Harness;
use filmcraft_engine::Session;
use filmcraft_project::{ClipId, ParamValue};
use filmcraft_ui_egui::{FilmcraftApp, control::ControlRequest};
use serde_json::{Value, json};
use std::sync::mpsc::{Sender, channel};

struct Driver {
    harness: Harness<'static, FilmcraftApp>,
    tx: Sender<ControlRequest>,
}

impl Driver {
    fn demo() -> Self {
        let mut session = Session::default();
        session.execute("file.openDemoProject", json!({})).unwrap();
        let (tx, rx) = channel();
        let app = FilmcraftApp::new(session).with_control(rx);
        let harness = Harness::builder().with_size(egui::vec2(1600.0, 980.0)).with_max_steps(10000).build_eframe(move |_| app);
        let mut d = Self { harness, tx };
        d.frames(5);
        d.exec("playhead.set", json!({"seconds":2}));
        d
    }
    fn frames(&mut self, n: usize) {
        for _ in 0..n {
            let ctx = self.harness.ctx.clone();
            let mut raw = std::mem::take(self.harness.input_mut());
            eframe::App::raw_input_hook(self.harness.state_mut(), &ctx, &mut raw);
            *self.harness.input_mut() = raw;
            self.harness.step();
        }
    }
    fn call(&mut self, method: &str, params: Value) -> Value {
        let (request, reply) = ControlRequest::new(method, params.clone());
        self.tx.send(request).unwrap();
        for _ in 0..600 {
            self.frames(1);
            if let Ok(v) = reply.try_recv() {
                assert_eq!(v["ok"], true, "{method} {params}: {v}");
                return v["result"].clone();
            }
        }
        panic!("no reply: {method} {params}");
    }
    fn exec(&mut self, command: &str, params: Value) -> Value {
        self.call("engine.execute", json!({"command":command,"params":params}))
    }
    fn rect(&mut self, id: &str) -> egui::Rect {
        self.frames(3);
        let all = self.call("ui.elements", json!({"prefix":id}));
        let e = all.as_array().unwrap().iter().find(|v| v["id"] == id).unwrap_or_else(|| panic!("missing {id}: {all}"));
        let r = e["rect"].as_array().unwrap();
        egui::Rect::from_min_size(
            egui::pos2(r[0].as_f64().unwrap() as f32, r[1].as_f64().unwrap() as f32),
            egui::vec2(r[2].as_f64().unwrap() as f32, r[3].as_f64().unwrap() as f32),
        )
    }
    fn drag(&mut self, from: egui::Pos2, to: egui::Pos2) {
        self.call("ui.drag", json!({"from":{"x":from.x,"y":from.y},"to":{"x":to.x,"y":to.y},"steps":6,"modifiers":{"command":true}}));
        self.frames(12);
    }
    fn param(&self, clip: u64, param: &str) -> ParamValue {
        let s = &self.harness.state().session;
        let (_, it) = s.active_sequence().unwrap().find_item(ClipId(clip)).unwrap();
        it.effect("motion").unwrap().param(param).unwrap().value_at(it.source_time_at(s.playhead()))
    }
    fn add(&mut self, item: u64, position: [f64; 2], scale: f64) -> u64 {
        let r = self.exec("timeline.place", json!({"item":item,"track":"new","seconds":0,"video":true,"audio":false}));
        let clip = r["clips"][0].as_u64().unwrap();
        self.exec("motion.set", json!({"clip":clip,"position":position,"scale":scale,"scaleWidth":scale}));
        self.exec("timeline.select", json!({"clips":[]}));
        self.frames(3);
        clip
    }
    fn exercise(&mut self, clip: u64) {
        let body = self.rect(&format!("program.media.{clip}"));
        self.call("ui.click", json!({"x":body.center().x,"y":body.center().y}));
        self.frames(4);
        assert!(self.harness.state().session.state.selection.contains(&ClipId(clip)));
        self.rect(&format!("program.media.{clip}.handle.0"));
        let before = self.param(clip, "position").as_vec2().unwrap();
        let count = self.harness.state().session.history.undo.len();
        self.drag(body.center(), body.center() + egui::vec2(-43.0, 37.0));
        let moved = self.param(clip, "position").as_vec2().unwrap();
        assert!(moved.x < before.x - 10.0 && moved.y > before.y + 10.0, "{before:?} -> {moved:?}");
        assert_eq!(self.harness.state().session.history.undo.len(), count + 1);
        self.exec("edit.undo", json!({}));
        assert_eq!(self.param(clip, "position").as_vec2().unwrap(), before);
        self.exec("edit.redo", json!({}));
        assert_eq!(self.param(clip, "position").as_vec2().unwrap(), moved);

        let pin = self.rect(&format!("program.media.{clip}.handle.0")).center();
        let handle = self.rect(&format!("program.media.{clip}.handle.2")).center();
        let old_scale = self.param(clip, "scale").as_f64().unwrap();
        let count = self.harness.state().session.history.undo.len();
        self.drag(handle, pin + (handle - pin) * 0.7);
        let new_scale = self.param(clip, "scale").as_f64().unwrap();
        assert!((new_scale / old_scale - 0.7).abs() < 0.02, "{old_scale} -> {new_scale}");
        assert_eq!(self.harness.state().session.history.undo.len(), count + 1);
        let new_pin = self.rect(&format!("program.media.{clip}.handle.0")).center();
        assert!((new_pin - pin).length() < 1.5, "opposite corner stays put: {pin:?} -> {new_pin:?}");
        self.exec("edit.undo", json!({}));
        assert_eq!(self.param(clip, "scale").as_f64().unwrap(), old_scale);
        assert_eq!(self.param(clip, "position").as_vec2().unwrap(), moved);
    }
}

#[test]
fn video_overlay_can_be_selected_moved_resized_and_undone() {
    let mut d = Driver::demo();
    let seq = d.harness.state().session.active_sequence().unwrap();
    let item = seq.video_tracks[0].items[0].item.0;
    let base = seq.video_tracks[0].items[0].id.0;
    let unchanged = d.param(base, "position");
    let clip = d.add(item, [1150.0, 420.0], 35.0);
    d.exercise(clip);
    assert_eq!(d.param(base, "position"), unchanged);
}

#[test]
fn imported_still_above_video_has_the_same_handles() {
    let mut d = Driver::demo();
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/fixtures/media-transform");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("synthetic-still.png");
    image::RgbaImage::from_pixel(600, 400, image::Rgba([40, 160, 220, 255])).save(&path).unwrap();
    let item = d.exec("file.import", json!({"paths":[path],"imageSequence":false}))["items"][0].as_u64().unwrap();
    let clip = d.add(item, [1150.0, 420.0], 100.0);
    d.exercise(clip);
}

#[test]
fn rotated_nonuniform_video_preserves_its_ratio_and_pinned_corner() {
    let mut d = Driver::demo();
    let item = d.harness.state().session.active_sequence().unwrap().video_tracks[0].items[0].item.0;
    let clip = d.add(item, [1100.0, 450.0], 30.0);
    d.exec("effects.setParam", json!({"clip":clip,"param":"rotation","value":23}));
    d.exec("effects.setParam", json!({"clip":clip,"param":"uniform_scale","value":false}));
    d.exec("effects.setParam", json!({"clip":clip,"param":"scale_width","value":45}));
    d.exercise(clip);
    assert!((d.param(clip, "scale_width").as_f64().unwrap() / d.param(clip, "scale").as_f64().unwrap() - 1.5).abs() < 1e-6);
}

#[test]
fn graphics_and_masks_keep_their_input_priority_and_locked_tracks_have_no_handles() {
    let mut d = Driver::demo();
    let item = d.harness.state().session.active_sequence().unwrap().video_tracks[0].items[0].item.0;
    let clip = d.add(item, [960.0, 540.0], 50.0);
    let graphic = d.exec("graphics.newText", json!({"text":"Foreground title","position":[960,540],"size":60}))["clip"].as_u64().unwrap();
    let text = d.rect(&format!("program.layer.{graphic}.0"));
    d.call("ui.click", json!({"x":text.center().x,"y":text.center().y}));
    d.frames(4);
    assert!(d.harness.state().session.state.selection.contains(&ClipId(graphic)));
    d.exec("masks.add", json!({"clip":clip,"effect":"opacity","shape":"ellipse"}));
    d.frames(4);
    assert!(d.harness.state().session.state.selected_mask.is_some());
    let elements = d.call("ui.elements", json!({"prefix":format!("program.media.{clip}")}));
    assert!(elements.as_array().unwrap().is_empty(), "mask editing owns the preview");
    d.exec("masks.select", json!({"none":true}));
    // Select the overlay on the timeline, then lock its actual track.
    let track = d.harness.state().session.active_sequence().unwrap().find_item(ClipId(clip)).unwrap().0.0;
    d.exec("timeline.setTrack", json!({"track":track,"locked":true}));
    d.exec("timeline.select", json!({"clips":[clip]}));
    d.frames(4);
    let elements = d.call("ui.elements", json!({"prefix":format!("program.media.{clip}")}));
    assert!(elements.as_array().unwrap().is_empty());
}

#[test]
fn option_drag_duplicates_both_video_and_still_above_the_original_in_one_undo() {
    let mut d = Driver::demo();
    let q = d.harness.state().session.active_sequence().unwrap();
    assert_eq!((q.video_tracks.len(), q.audio_tracks.len()), (10, 10));
    let movie = q.video_tracks[0].items[0].item.0;
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/fixtures/media-transform/option-still.png");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    image::RgbaImage::from_pixel(600, 400, image::Rgba([220, 160, 40, 255])).save(&path).unwrap();
    let still = d.exec("file.import", json!({"paths":[path],"imageSequence":false}))["items"][0].as_u64().unwrap();
    for item in [movie, still] {
        let clip = d.add(item, [1150.0, 420.0], 50.0);
        d.exec("timeline.select", json!({"clips":[clip]}));
        let from = d.rect(&format!("program.media.{clip}")).center();
        let before = d.harness.state().session.project.clone();
        let n = d.harness.state().session.history.undo.len();
        d.call("ui.drag", json!({"from":{"x":from.x,"y":from.y},"to":{"x":from.x-55.0,"y":from.y+35.0},"steps":6,"modifiers":{"alt":true,"command":true}}));
        d.frames(12);
        let s = &d.harness.state().session;
        let copy = s.state.selection[0];
        assert_ne!(copy, ClipId(clip));
        let q = s.active_sequence().unwrap();
        let (track, original) = q.find_item(ClipId(clip)).unwrap();
        let index = q.video_tracks.iter().position(|tr| tr.id == track).unwrap();
        let copied = q.find_item(copy).unwrap().1;
        assert_eq!(q.video_tracks[index + 1].items[0].id, copy);
        assert_eq!((copied.item, copied.start, copied.duration, copied.source_in), (original.item, original.start, original.duration, original.source_in));
        assert_eq!(copied.effect("motion").unwrap().param("scale"), original.effect("motion").unwrap().param("scale"));
        assert_ne!(copied.effect("motion").unwrap().param("position"), original.effect("motion").unwrap().param("position"));
        assert_eq!(s.history.undo.len(), n + 1);
        d.exec("edit.undo", json!({}));
        assert_eq!(d.harness.state().session.project, before);
    }
}

#[test]
fn selected_nonconcurrent_texts_follow_live_and_option_drag_duplicates_only_the_grabbed_text() {
    let mut d = Driver::demo();
    let a = d.exec("graphics.newText", json!({"newClip":true,"text":"Visible title","position":[600,400],"size":80,"time":0,"seconds":3}))["clip"]
        .as_u64()
        .unwrap();
    let b = d.exec(
        "graphics.newText",
        json!({"newClip":true,"text":"Later title","position":[950,650],"size":55,"seconds":3,"time":filmcraft_time::Tick::from_seconds_f64(5.0).0}),
    )["clip"]
        .as_u64()
        .unwrap();
    d.exec("timeline.select", json!({"clips":[b,a]}));
    d.call("ui.set", json!({"tool":"Selection"}));
    let from = d.rect(&format!("program.layer.{a}.0")).center();
    d.call("ui.click", json!({"x":from.x,"y":from.y}));
    assert_eq!(d.harness.state().session.state.selection.len(), 2);
    let before = d.harness.state().session.project.clone();
    let n = d.harness.state().session.history.undo.len();
    let m = egui::Modifiers { command: true, ..Default::default() };
    let to = from + egui::vec2(70.0, 45.0);
    d.harness.input_mut().events.push(egui::Event::ModifiersChanged(m));
    d.harness
        .input_mut()
        .events
        .extend([egui::Event::PointerMoved(from), egui::Event::PointerButton { pos: from, button: egui::PointerButton::Primary, pressed: true, modifiers: m }]);
    d.frames(1);
    d.harness.input_mut().events.push(egui::Event::PointerMoved(to));
    d.frames(3);
    let pa = d.exec("graphics.list", json!({"clip":a}))["layers"][0]["position"].clone();
    let pb = d.exec("graphics.list", json!({"clip":b}))["layers"][0]["position"].clone();
    assert_ne!(pa, json!([600.0, 400.0]));
    assert_eq!(pa, pb, "all selected text positions update before releasing the mouse");
    d.harness.input_mut().events.push(egui::Event::PointerButton { pos: to, button: egui::PointerButton::Primary, pressed: false, modifiers: m });
    d.frames(3);
    assert_eq!(d.harness.state().session.history.undo.len(), n + 1);
    assert_eq!(d.harness.state().session.state.selection.len(), 2);
    d.exec("edit.undo", json!({}));
    assert_eq!(d.harness.state().session.project, before);
    d.exec("timeline.select", json!({"clips":[a,b]}));
    let from = d.rect(&format!("program.layer.{a}.0")).center();
    let n = d.harness.state().session.history.undo.len();
    d.call("ui.drag", json!({"from":{"x":from.x,"y":from.y},"to":{"x":from.x+60.0,"y":from.y+40.0},"steps":6,"modifiers":{"alt":true,"command":true}}));
    d.frames(12);
    let copy = d.harness.state().session.state.selection[0].0;
    assert!(copy != a && copy != b);
    let original = d.exec("graphics.list", json!({"clip":a}));
    let copied = d.exec("graphics.list", json!({"clip":copy}));
    assert_eq!(copied["start"], original["start"]);
    assert_eq!(copied["duration"], original["duration"]);
    assert_eq!(copied["layers"][0]["text"], original["layers"][0]["text"]);
    assert_eq!(copied["layers"][0]["scale"], original["layers"][0]["scale"]);
    assert_eq!(original["layers"][0]["position"], json!([600.0, 400.0]));
    assert_eq!(d.harness.state().session.history.undo.len(), n + 1);
    d.exec("edit.undo", json!({}));
    assert_eq!(d.harness.state().session.project, before);
}
