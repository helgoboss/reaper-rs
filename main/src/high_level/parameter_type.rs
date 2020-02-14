// TODO This is probably out of scope for reaper-rs and should go just to ReaLearn.
//  Rust seems much better suited for this kind of extensibility than C++, so we don't need to
//  implement lots of interfaces in this crate just to make all developers happy.
#[derive(Debug, Eq, PartialEq)]
pub enum ParameterType {
    Action
}