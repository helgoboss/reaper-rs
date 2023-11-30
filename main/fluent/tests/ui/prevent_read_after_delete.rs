use reaper_fluent::*;

fn main() {
    let model = Reaper::get().model();
    let mut project = model.current_project();
    let track = project.tracks().next().unwrap();
    project.delete_track(track.raw());
    track.guid();
}
