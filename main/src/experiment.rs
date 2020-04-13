#[derive(Default)]
pub struct Reaper {
    pub pointers: ReaperFunctionPointers,
}

impl Reaper {
    pub unsafe fn PluginWantsAlwaysRunFx(&self, amt: ::std::os::raw::c_int) -> i32 {
        match self.pointers.PluginWantsAlwaysRunFx {
            None => panic!(format!(
                "Attempt to use a REAPER function that has not been loaded: {}",
                stringify!(PluginWantsAlwaysRunFx)
            )),
            Some(f) => f(amt),
        }
    }
}

#[derive(Default)]
pub struct ReaperFunctionPointers {
    pub PluginWantsAlwaysRunFx: Option<unsafe extern "C" fn(amt: ::std::os::raw::c_int) -> i32>,
}
