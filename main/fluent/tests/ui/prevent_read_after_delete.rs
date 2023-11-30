use reaper_fluent::*;

fn main() {
    let mut model = Reaper::get().model_mut();
    let project = model.current_project();
    let track = project.tracks().next().unwrap();
    let raw = track.raw();
    model.current_project_mut().delete_track(raw);
    track.guid();
}
