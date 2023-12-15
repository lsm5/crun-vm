// SPDX-License-Identifier: GPL-2.0-or-later

use std::ffi::OsStr;
use std::io::{self, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde::Deserialize;

pub fn find_single_file_in_directories<I, P>(dir_paths: I) -> io::Result<PathBuf>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    let mut candidate = None;

    for dir_path in dir_paths {
        let dir_path = dir_path.as_ref();

        if dir_path.is_dir() {
            for entry in dir_path.read_dir()? {
                let e = entry?;

                if e.file_type()?.is_file() {
                    if candidate.is_some() {
                        return Err(io::Error::other("more than one file found"));
                    } else {
                        candidate = Some(e.path());
                    }
                }
            }
        }
    }

    if let Some(path) = candidate {
        Ok(path)
    } else {
        Err(io::Error::other("no files found"))
    }
}

/// Unpacks the crun-qemu runner container image's root filesystem into the given directory.
///
/// TODO: Embedding this root filesystem into the crun-qemu executable and unpacking it every time
/// the container engine asks us to create a container is handy for development but inefficient and
/// ugly. It should instead be installed alongside the crun-qemu runtime as a directory somewhere on
/// the system.
pub fn extract_runner_root_into(dir_path: impl AsRef<Path>) -> io::Result<()> {
    let tar_bytes: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/runner.tar"));
    let tar_reader = BufReader::new(tar_bytes);

    tar::Archive::new(tar_reader).unpack(dir_path)?;

    Ok(())
}

pub fn get_image_format(image_path: impl AsRef<Path>) -> io::Result<String> {
    let output = Command::new("qemu-img")
        .arg("info")
        .arg("--output=json")
        .arg(image_path.as_ref().as_os_str())
        .stdout(Stdio::piped())
        .output()?;

    if !output.status.success() {
        return Err(io::Error::other("qemu-img failed"));
    }

    #[derive(Deserialize)]
    struct ImageInfo {
        format: String,
    }

    let info: ImageInfo = serde_json::from_slice(&output.stdout)?;

    Ok(info.format)
}

pub fn create_overlay_image(
    overlay_image_path: impl AsRef<Path>,
    backing_image_path: impl AsRef<Path>,
) -> io::Result<()> {
    let backing_image_format = get_image_format(backing_image_path.as_ref())?;

    let status = Command::new("qemu-img")
        .arg("create")
        .arg("-q")
        .arg("-f")
        .arg("qcow2")
        .arg("-F")
        .arg(backing_image_format)
        .arg("-b")
        .arg(backing_image_path.as_ref())
        .arg(overlay_image_path.as_ref())
        .spawn()?
        .wait()?;

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other("qemu-img failed"))
    }
}

/// Run `crun`.
///
/// `crun` will inherit this process' standard streams.
///
/// TODO: It may be better to use libcrun directly, although its public API purportedly isn't in
/// great shape: https://github.com/containers/crun/issues/1018
pub fn crun<I, S>(args: I) -> io::Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let status = Command::new("crun").args(args).spawn()?.wait()?;

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other("crun failed"))
    }
}
