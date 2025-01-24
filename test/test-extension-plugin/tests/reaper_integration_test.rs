#![cfg(feature = "run-reaper-integration-test")]
use fs_extra::dir::CopyOptions;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;
use std::{fs, io};
use wait_timeout::ChildExt;

use anyhow::{bail, Context, Result};

const REAPER_VERSION: &str = "7.30";

#[test]
fn run_reaper_integration_test() -> Result<()> {
    let target_dir_path = std::env::current_dir()?
        .join("../../target")
        .canonicalize()?;
    let reaper_download_dir_path = target_dir_path.join("reaper");
    if cfg!(target_os = "macos") {
        run_on_macos(&target_dir_path, &reaper_download_dir_path)
    } else if cfg!(target_os = "linux") {
        run_on_linux(&target_dir_path, &reaper_download_dir_path)
    } else {
        println!("Skipping headless reaper-rs integration tests because not supported on this OS");
        return Ok(());
    }
}

fn run_on_linux(target_dir_path: &Path, reaper_download_dir_path: &Path) -> Result<()> {
    let reaper_home_path = setup_reaper_for_linux(reaper_download_dir_path)?;
    install_plugin(&target_dir_path, &reaper_home_path)?;
    let reaper_executable = reaper_home_path.join("reaper");
    run_integration_test_in_reaper(&reaper_executable)?;
    Ok(())
}

fn run_on_macos(target_dir_path: &Path, reaper_download_dir_path: &Path) -> Result<()> {
    let reaper_home_path = setup_reaper_for_macos(reaper_download_dir_path)?;
    install_plugin(&target_dir_path, &reaper_home_path)?;
    let reaper_executable = reaper_home_path.join("REAPER.app/Contents/MacOS/REAPER");
    run_integration_test_in_reaper(&reaper_executable)?;
    Ok(())
}

fn install_plugin(target_dir_path: &Path, reaper_home_path: &Path) -> Result<()> {
    let extension = if cfg!(target_os = "macos") {
        "dylib"
    } else {
        "so"
    };
    let source_path = target_dir_path
        .join("debug")
        .join(format!("libreaper_test_extension_plugin.{}", extension));
    let target_path = reaper_home_path
        .join("UserPlugins")
        .join(format!("reaper_test_extension_plugin.{}", extension));
    fs::create_dir_all(target_path.parent().context("no parent")?)?;
    println!("Copying plug-in to {:?}...", &target_path);
    fs::copy(&source_path, &target_path)?;
    Ok(())
}

fn run_integration_test_in_reaper(reaper_executable: &Path) -> Result<()> {
    println!("Starting REAPER ({:?})...", &reaper_executable);
    let mut child = Command::new(reaper_executable)
        .env("RUN_REAPER_RS_INTEGRATION_TEST", "true")
        // .arg("-splashlog")
        // .arg("splash.log")
        .spawn()?;
    let exit_status = child.wait_timeout(Duration::from_secs(120))?;
    let exit_status = match exit_status {
        None => {
            child.kill()?;
            bail!("REAPER didn't exit in time (maybe integration test has not started at all)",);
        }
        Some(s) => s,
    };
    if exit_status.success() {
        // Integration test instance returned successfully
        return Ok(());
    }
    match exit_status.code() {
        Some(172) => {
            bail!("Integration test failed due to failing test step");
        }
        Some(x) => {
            bail!("Integration test failed because REAPER process returned exit code {x}");
        }
        None => {
            bail!("Integration test failed because REAPER process didn't return any exit code. Unix signal: {:?}", exit_status.unix_signal());
        }
    }
}

/// Returns path of REAPER home
fn setup_reaper_for_linux(reaper_download_dir_path: &Path) -> Result<PathBuf> {
    let (tarball_suffix, base_dir) = if cfg!(target_arch = "aarch64") {
        ("_linux_aarch64.tar.xz", "reaper_linux_aarch64")
    } else if cfg!(target_arch = "x86_64") {
        ("_linux_x86_64.tar.xz", "reaper_linux_x86_64")
    } else {
        bail!("Linux architecture not supported");
    };
    let reaper_home_path = reaper_download_dir_path.join(base_dir).join("REAPER");
    if reaper_home_path.exists() {
        return Ok(reaper_home_path);
    }
    let reaper_tarball_path = reaper_download_dir_path.join("reaper.tar.xz");
    if !reaper_tarball_path.exists() {
        let url = get_reaper_download_url(REAPER_VERSION, tarball_suffix)?;
        println!("Downloading from {url} REAPER to {reaper_tarball_path:?}...");
        download(&url, &reaper_tarball_path)?;
    }
    println!("Unpacking REAPER tarball...");
    unpack_tar_xz(&reaper_tarball_path, &reaper_download_dir_path)?;
    write_reaper_config(&reaper_home_path)?;
    println!("REAPER home directory is {:?}", &reaper_home_path);
    Ok(reaper_home_path)
}

/// Returns path of REAPER home
fn setup_reaper_for_macos(reaper_download_dir_path: &Path) -> Result<PathBuf> {
    let reaper_home_path = reaper_download_dir_path.join("reaper");
    if reaper_home_path.exists() {
        return Ok(reaper_home_path);
    }
    let dmg_path = reaper_download_dir_path.join("reaper.dmg");
    if !dmg_path.exists() {
        let url = get_reaper_download_url(REAPER_VERSION, "_universal.dmg")?;
        println!("Downloading REAPER from {url} to {dmg_path:?}...");
        download(&url, &dmg_path)?;
    }
    println!("Unpacking REAPER DMG...");
    let mount_dir = mount_dmg(&dmg_path)?;
    println!("Copying from mount...");
    fs::create_dir_all(&reaper_home_path)?;
    fs_extra::dir::copy(
        mount_dir.join("REAPER.app"),
        &reaper_home_path,
        &CopyOptions {
            overwrite: false,
            skip_exist: false,
            buffer_size: 0,
            copy_inside: false,
            depth: 0,
            ..Default::default()
        },
    )?;
    println!("Unmount DMG...");
    unmount_dir_macos(&mount_dir)?;
    write_reaper_config(&reaper_home_path)?;
    // remove_rewire_plugin_macos_bundle(&reaper_home_path)?;
    println!("REAPER home directory is {:?}", &reaper_home_path);
    Ok(reaper_home_path)
}

fn write_reaper_config(reaper_home_path: &Path) -> Result<()> {
    println!("Writing REAPER configuration...");
    let content = r#"
[audioconfig]
; For dummy audio on Windows
mode=4

[REAPER]
; Not scanning installed VST instruments.
; This still does some scanning because REAPER auto-adds some external folders :/
vstpath=
vstpath_arm64=
;This even doesn't scan internal VSTs :/
;vst_scan=2
; For dummy audio on Linux
linux_audio_mode=2
; For <none> audio on macOS
coreaudiobs=512
coreaudioindevnew=<none>
coreaudiooutdevnew=<none>
"#;
    fs::write(reaper_home_path.join("reaper.ini"), content)?;
    Ok(())
}

#[allow(dead_code)]
fn remove_rewire_plugin_macos_bundle(reaper_home_path: &Path) -> Result<()> {
    println!("Removing Rewire plug-in (because it makes REAPER get stuck on headless macOS)...");
    let dir = reaper_home_path.join("REAPER.app/Contents/Plugins/ReWire.bundle");
    fs::remove_dir_all(&dir).with_context(|| dir.to_string_lossy().to_string())?;
    Ok(())
}

fn download(url: &str, dest_file_path: &Path) -> Result<()> {
    let mut response = reqwest::blocking::get(url)?;
    fs::create_dir_all(
        dest_file_path
            .parent()
            .context("download destination path must be absolute")?,
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

fn unmount_dir_macos(mount: &Path) -> Result<()> {
    if !Command::new("hdiutil")
        .arg("detach")
        .arg(mount)
        .spawn()?
        .wait()?
        .success()
    {
        bail!("Detaching the mounted image failed");
    }
    Ok(())
}

fn mount_dmg(dmg: &Path) -> Result<PathBuf> {
    let dir = dmg.parent().context("dmg has no parent dir")?;
    let mount_dir = dir.join("mounted-dmg");
    let cdr = dir.join("reaper.cdr");
    if !Command::new("hdiutil")
        .arg("convert")
        .arg("-quiet")
        .arg(dmg)
        .arg("-format")
        .arg("UDTO")
        .arg("-o")
        .arg(&cdr)
        .spawn()?
        .wait()?
        .success()
    {
        bail!("conversion to CDR image failed");
    }
    if !Command::new("hdiutil")
        .arg("attach")
        .arg(&cdr)
        .arg("-mountpoint")
        .arg(&mount_dir)
        .spawn()?
        .wait()?
        .success()
    {
        bail!("mount not successful");
    }
    Ok(mount_dir)
}

fn get_reaper_download_url(version: &str, suffix: &str) -> Result<String> {
    let (major, _) = version
        .split_once('.')
        .context("REAPER version should contain dot")?;
    let dot_less_version = version.replace('.', "");
    Ok(format!(
        "https://www.reaper.fm/files/{major}.x/reaper{dot_less_version}{suffix}"
    ))
}
