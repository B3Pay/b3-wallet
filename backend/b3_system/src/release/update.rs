use b3_shared::{
    b3_trap,
    types::{Blob, Version},
};
use b3_system_lib::{
    error::SystemError,
    store::{with_release_mut, with_releases_mut, with_version_release_mut},
    types::{LoadRelease, Release, ReleaseArgs},
};
use ic_cdk::{export::candid::candid_method, update};

use crate::guards::caller_is_controller;

#[candid_method(update)]
#[update(guard = "caller_is_controller")]
fn update_release(release_args: ReleaseArgs) -> Result<(), SystemError> {
    let version = release_args.version.clone();

    with_version_release_mut(version, |vrs| {
        vrs.update(release_args);
    })
}

#[candid_method(update)]
#[update(guard = "caller_is_controller")]
fn load_release(blob: Blob, release_args: ReleaseArgs) -> Result<LoadRelease, SystemError> {
    let version = release_args.version.clone();

    let release_index =
        with_releases_mut(|rs| match rs.iter().position(|r| r.version == version) {
            Some(index) => index,
            None => {
                let release = Release::new(release_args);
                rs.push(release);

                rs.len() - 1
            }
        });

    let total = with_release_mut(release_index, |r| r.load_wasm(&blob))
        .unwrap_or_else(|e| b3_trap(e))
        .unwrap_or_else(|e| b3_trap(e));

    let chunks = blob.len();

    Ok(LoadRelease {
        version,
        chunks,
        total,
    })
}

#[candid_method(update)]
#[update(guard = "caller_is_controller")]
pub fn remove_release(version: Version) -> Result<Release, SystemError> {
    with_releases_mut(|rs| match rs.iter().position(|r| r.version == version) {
        Some(index) => Ok(rs.remove(index)),
        None => Err(SystemError::ReleaseNotFound),
    })
}

#[candid_method(update)]
#[update(guard = "caller_is_controller")]
fn remove_latest_release() {
    with_releases_mut(|rs| {
        rs.pop();
    });
}

#[candid_method(update)]
#[update(guard = "caller_is_controller")]
fn deprecate_release(version: Version) -> Result<(), SystemError> {
    with_version_release_mut(version, |vrs| {
        vrs.deprecate();
    })
}
