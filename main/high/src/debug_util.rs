use crate::{determine_module_info, Reaper};
use backtrace::Symbol;
use regex::Regex;

/// Attempts to enrich a backtrace produced by the reaper_high default panic hook with symbol
/// information.
///
/// The problem is that release builds are usually delivered and executed without symbol
/// information, so if a panic occurs and the user sends an error report, the backtrace just
/// contains address numbers. This seems to be especially bad on Windows. In order to find out
/// about the function names and maybe even lines of code, it's necessary to look those addresses
/// up in the symbol database belonging to that release build. The result might still not be very
/// satisfying, e.g. because of compiler optimizations. But it's worth a try.
///
/// The alternative would be to create a minidump at error time, but AFAIK that can only be done by
/// aborting the REAPER process - and this is something which we don't want for sure!  
///
/// - Windows only.
/// - Only works if executed in exactly the same plug-in build (OS, architecture, plug-in version)
///   as the one in which the error occurred.
/// - Corresponding PDB file must be loaded.
/// - Expects the first two hex numbers to be the module base address and the module size. That
///   should always be the case in Windows builds. This function takes such a backtrace and attempts
///   to print any results to the REAPER ReaScript console. It starts by just finding all hex
///   numbers starting with `0x` and processing them. So you can easily preprocess the input if it
///   doesn't match this spec.
pub fn resolve_symbols_from_text(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    let regex = Regex::new(r"\b0x[0-9a-f]+\b").unwrap();
    let hex_strings = regex
        .find_iter(text)
        .map(|m| m.as_str().trim().trim_start_matches("0x"));
    let hex_numbers: Result<Vec<usize>, _> = hex_strings
        .map(|s| usize::from_str_radix(s, 16).map_err(|_| "invalid address"))
        .collect();
    let hex_numbers = hex_numbers?;
    let their_module_base_address = *hex_numbers
        .first()
        .ok_or("Module base address missing in error report")?;
    let their_module_size = *hex_numbers
        .get(1)
        .ok_or("Module size missing in error report")?;
    resolve_multiple_symbols(
        their_module_base_address,
        their_module_size,
        &hex_numbers[2..],
    )?;
    Ok(())
}

fn resolve_multiple_symbols(
    their_module_base_address: usize,
    their_module_size: usize,
    addresses: &[usize],
) -> Result<(), Box<dyn std::error::Error>> {
    log(format!(
        "Attempting to resolve symbols for {} addresses\n\
        ==============================================
        ",
        addresses.len()
    ));
    let our_module_info = determine_module_info()
        .ok_or("Couldn't get own module info ... maybe we are not on Windows?")?;
    if let Some(our_module_size) = our_module_info.size {
        if our_module_size != their_module_size {
            warn(format!(
                "Module sizes deviating (ours: {:x}, theirs: {:x})",
                our_module_size, their_module_size
            ));
        }
    } else {
        warn("Couldn't get own module size".to_string());
    }
    let their_module_until_address = their_module_base_address + their_module_size;
    for (i, &a) in addresses.iter().enumerate() {
        let rich_address = if a >= their_module_base_address && a < their_module_until_address {
            let relative = a - their_module_base_address;
            Address::Internal {
                their_absolute: a,
                relative,
                our_absolute: our_module_info.base_address + relative,
            }
        } else {
            Address::External(a)
        };
        resolve_one_of_multiple_symbols(i, rich_address);
    }
    Ok(())
}

enum Address {
    /// Address which belongs to our module.
    Internal {
        /// Absolute address within the process in which this backtrace was generated.
        ///
        /// their_absolute = their_module_base_address + relative
        their_absolute: usize,
        /// Relative address starting from module base address.
        ///
        /// I think that's sometimes called "offset".
        relative: usize,
        /// Absolute address within our process.
        our_absolute: usize,
    },
    /// Address which doesn't belong to this module but e.g. to REAPER itself.
    External(usize),
}

impl Address {
    fn address_to_be_resolved(&self) -> usize {
        use Address::*;
        match self {
            Internal { our_absolute, .. } => *our_absolute,
            External(a) => *a,
        }
    }

    fn their_absolute(&self) -> usize {
        use Address::*;
        match self {
            Internal { their_absolute, .. } => *their_absolute,
            External(a) => *a,
        }
    }

    fn relative(&self) -> Option<usize> {
        use Address::*;
        match self {
            Internal { relative, .. } => Some(*relative),
            External(_) => None,
        }
    }

    fn our_absolute(&self) -> Option<usize> {
        use Address::*;
        match self {
            Internal { our_absolute, .. } => Some(*our_absolute),
            External(_) => None,
        }
    }
}

fn resolve_one_of_multiple_symbols(i: usize, address: Address) {
    backtrace::resolve(address.address_to_be_resolved() as _, |sym| {
        log(format!(
            "{i}: {their_absolute:x}\n\
            -----------------------------------
            \n\
            Relative: {relative:x?}\n\
            Ours absolute: {our_absolute:x?}\n\
            \n\
            {sym}\n\
            \n\
            ",
            i = i,
            their_absolute = address.their_absolute(),
            relative = address.relative(),
            our_absolute = address.our_absolute(),
            sym = format_symbol_terse(sym)
        ));
    });
}

fn format_symbol_terse(sym: &Symbol) -> String {
    let segments: Vec<String> = vec![
        sym.addr().map(|a| format!("{:x?}", a as isize)),
        sym.name().map(|n| n.to_string()),
        sym.filename().map(|p| {
            format!(
                "{}{}",
                p.to_string_lossy(),
                sym.lineno()
                    .map(|n| format!(" (line {})", n))
                    .unwrap_or_else(|| "".to_string())
            )
        }),
    ]
    .into_iter()
    .flatten()
    .collect();
    segments.join("\n")
}

fn warn(msg: String) {
    log(format!("WARNING: {}", msg))
}

fn log(msg: String) {
    Reaper::get().show_console_msg(format!("{}\n", msg));
}
