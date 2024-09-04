// This file contains some function definitions which for some reason are not generated via REAPER action.
// Source: https://www.reaper.fm/sdk/plugin/reaper_plugin_functions.h

// if you wish to access MIDI inputs/outputs that are opened via Prefs/MIDI, you may do so, but ONLY if in the audio thread,
// specifically in a hook installed by Audio_RegHardwareHook, or if in a VST/etc and IsInRealTimeAudio() returns TRUE.
// The API:

midi_Input* (* GetMidiInput)(int idx);
midi_Output* (* GetMidiOutput)(int idx);

/*
    You should call the above GetMidi*put() before you use them, to verify the device is still open.
    Do NOT call midi_Input::SwapBufs(), but you can call GetReadBuf() to peek in the MIDI input.
    Do NOT call midi_Output::BeginBlock()/EndBlock() in this mode, just Send()/SendMsg().

*/

// fxDoReaperPresetAction(parentid, "preset name",0);  // will save the preset to 'preset name'
// NOT USED YET
int (* fxDoReaperPresetAction)(void* fx, const char* name, int flag);

// - extra_flags can have 1 set to signify "do not refresh the toolbar/menus" -- if you do a batch of updates you'd set 1 for everything except the final one
// - all changes do not persist. If the user customizes the menu after your change was added, then it does persist, because the user customization makes it stick.
// - toolbarflags: &1= animate if enabled-state, &2=animate if disabled-state, &0x7f8 is the animation mode
// - returns true on success or false on failure
bool (* AddCustomMenuOrToolbarItem)(const char* menuname,
    int pos,
    int command_id,
    int toolbarflags,
    const char* str,
    const char* iconfn,
    int extra_flags);

// - extra_flags can have 1 set to signify "do not refresh the toolbar/menus" -- if you do a batch of updates you'd set 1 for everything except the final one
// - all changes do not persist. If the user customizes the menu after your change was added, then it does persist, because the user customization makes it stick.
// - returns true on success or false on failure
bool (* DeleteCustomMenuOrToolbarItem)(const char* menuname, int pos, int extra_flags);

// - returns true on success or false on failure
bool (* GetCustomMenuOrToolbarItem)(const char* menuname,
    int pos,
    int* commandOutOptional,
    int* toolbarFlagsOutOptional,
    const char** strOutOptional,
    const char** iconFnOutOptional);

// You can use this to step through times ahead of the current playback time, loopcnt will get updated on a loop or autoseek etc.
//
// double nextpos = old_pos;
// INT64 lc = GetPlayLoopCnt(proj, NULL);
// int ret = AdvancePlaybackPosition(proj, old_pos, &next_pos, &lc, 0.0 /* or srate */, NULL, NULL);
// ret 1 if looped sel, 2 if looped project, 4 if loopendskip, 8 if smoothseek, 16 if fade audition (all during this block)
// next_pos and lc updated so you can call again to look farther ahead
int (* AdvancePlaybackPosition)(ReaProject* __proj,
    double opos,
    double* npos,
    INT64* loopcnt,
    double srate,
    int* max_spls,
    int* sf);

// Not really sure what this does, but it should be used in combination with AdvancePlaybackPosition.
INT64 (* GetPlayLoopCnt)(ReaProject* __proj, void* something);