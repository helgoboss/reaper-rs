use crate::raw::gaccel_register_t;

// The contained raw string pointer doesn't do harm when sent to another thread.
unsafe impl Send for gaccel_register_t {}

// Same with Sync. We need runtime thread checks anyway to achieve safety.
unsafe impl Sync for gaccel_register_t {}
