// This file contains some function definitions which for some reason are not generated via REAPER action.
// Source: https://www.reaper.fm/sdk/plugin/reaper_plugin_functions.h

// if you wish to access MIDI inputs/outputs that are opened via Prefs/MIDI, you may do so, but ONLY if in the audio thread,
// specifically in a hook installed by Audio_RegHardwareHook, or if in a VST/etc and IsInRealTimeAudio() returns TRUE.
// The API:

midi_Input *(*GetMidiInput)(int idx);
midi_Output *(*GetMidiOutput)(int idx);

/*
    You should call the above GetMidi*put() before you use them, to verify the device is still open.
    Do NOT call midi_Input::SwapBufs(), but you can call GetReadBuf() to peek in the MIDI input.
    Do NOT call midi_Output::BeginBlock()/EndBlock() in this mode, just Send()/SendMsg().

*/