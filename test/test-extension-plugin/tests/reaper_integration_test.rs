use std::error::Error;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;
use std::{fs, io};
use wait_timeout::ChildExt;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[cfg(feature = "run-reaper-integration-test")]
#[cfg(target_os = "linux")]
#[test]
fn run_reaper_integration_test() {
    let target_dir_path = std::env::current_dir().unwrap().join("../../target");
    let reaper_download_dir_path = target_dir_path.join("reaper");
    let result = run_on_linux(&target_dir_path, &reaper_download_dir_path);
    result.expect("Running the integration test in REAPER failed");
}

fn run_on_linux(target_dir_path: &Path, reaper_download_dir_path: &Path) -> Result<()> {
    let reaper_home_path = setup_reaper_for_linux(reaper_download_dir_path)?;
    install_plugin(&target_dir_path, &reaper_home_path)?;
    let reaper_executable = reaper_home_path.join("reaper");
    run_integration_test_in_reaper(&reaper_executable)?;
    Ok(())
}

fn install_plugin(target_dir_path: &Path, reaper_home_path: &Path) -> Result<()> {
    let source_path = target_dir_path
        .join("debug")
        .join("libreaper_test_extension_plugin.so");
    let target_path = reaper_home_path
        .join("UserPlugins")
        .join("reaper_test_extension_plugin.so");
    fs::create_dir_all(target_path.parent().ok_or("no parent")?)?;
    println!("Copying plug-in to {:?}...", &target_path);
    fs::copy(&source_path, &target_path)?;
    Ok(())
}

fn run_integration_test_in_reaper(reaper_executable: &Path) -> Result<()> {
    println!("Starting REAPER ({:?})...", &reaper_executable);
    let mut child = Command::new(reaper_executable)
        .env("RUN_REAPER_RS_INTEGRATION_TEST", "true")
        .spawn()?;
    let exit_status = child.wait_timeout(Duration::from_secs(120))?;
    let exit_status = match exit_status {
        None => {
            child.kill()?;
            return Err(
                "REAPER didn't exit in time (maybe integration test has not started at all)",
            )?;
        }
        Some(s) => s,
    };
    if exit_status.success() {
        return Ok(());
    }
    let exit_code = exit_status
        .code()
        .ok_or("REAPER exited because of signal")?;
    if exit_code == 172 {
        Err("Integration test failed")?
    } else {
        Err(
            "REAPER exited unsuccessfully but neither because of signal nor because of failed \
            integration test",
        )?
    }
}

/// Returns path of REAPER home
fn setup_reaper_for_linux(reaper_download_dir_path: &Path) -> Result<PathBuf> {
    let reaper_home_path = reaper_download_dir_path.join("reaper_linux_x86_64/REAPER");
    if reaper_home_path.exists() {
        return Ok(reaper_home_path);
    }
    let reaper_tarball_path = reaper_download_dir_path.join("reaper-linux.tar.xz");
    if !reaper_tarball_path.exists() {
        println!("Downloading REAPER to ({:?})...", &reaper_tarball_path);
        download(
            "https://www.reaper.fm/files/6.x/reaper609_linux_x86_64.tar.xz",
            &reaper_tarball_path,
        )?;
    }
    println!("Unpacking REAPER tarball...");
    unpack_tar_xz(&reaper_tarball_path, &reaper_download_dir_path)?;
    println!("Activating REAPER portable mode...");
    fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(reaper_home_path.join("reaper.ini"))?;
    println!("REAPER home directory is {:?}", &reaper_home_path);
    Ok(reaper_home_path)
}

fn download(url: &str, dest_file_path: &Path) -> Result<()> {
    let mut response = reqwest::blocking::get(url)?;
    fs::create_dir_all(
        dest_file_path
            .parent()
            .ok_or("download destination path must be absolute")?,
    )?;
    let mut dest_file = fs::File::create(&dest_file_path)?;
    io::copy(&mut response, &mut dest_file)?;
    Ok(())
}

fn unpack_tar_xz(file_path: &Path, dest_dir_path: &Path) -> Result<()> {
    let tar_xz = File::open(file_path)?;
    let tar = xz2::read::XzDecoder::new(tar_xz);
    let mut archive = tar::Archive::new(tar);
    archive.unpack(dest_dir_path)?;
    Ok(())
}
