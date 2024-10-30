#[cfg(feature = "ultralight-resources")]
use std::env::var;
#[cfg(feature = "ultralight-resources")]
use std::fs::{self, DirEntry};
#[cfg(feature = "ultralight-resources")]
use std::path::Path;

fn main() {
    // ensure runtime resources exist - for examples & local tests
    #[cfg(not(feature = "ultralight-resources"))]
    std::env::var("ULTRALIGHT_RESOURCES_DIR").expect("If `--features ultralight-resources` is not passed, `ULTRALIGHT_RESOURCES_DIR` Must be set. \nSee README.md for more information");
    #[cfg(feature = "ultralight-resources")]
    {
        let mut possible_directories = Vec::new();
        let out = var("OUT_DIR").unwrap();
        // This allows it to work in this project but also other projects too
        let path = Path::new(&out)
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap();

        let target = Path::new(path).join("target");
        let debug_path = target.clone().join("debug");
        let release_path = target.clone().join("release");

        if let Ok(debug) = fs::exists(debug_path.clone()) {
            if debug {
                get_paths(
                    &mut possible_directories,
                    debug_path.join("build").to_str().unwrap().to_string(),
                )
            }
        } else if let Ok(release) = fs::exists(release_path.clone()) {
            if release {
                get_paths(
                    &mut possible_directories,
                    release_path.join("build").to_str().unwrap().to_string(),
                )
            }
        } else {
            panic!("Could not find either debug or release dirs")
        }

        assert!(!possible_directories.is_empty());

        let local_resources = Path::new(path).join("resources");

        for path in possible_directories {
            let resources_dir = path.path().join("out/ul-sdk/resources");
            if let Ok(resources) = fs::exists(resources_dir.clone()) {
                if resources {
                    if let Ok(local_resources_exist) = fs::exists(local_resources.clone()) {
                        if local_resources_exist {
                            fs::remove_file(local_resources.clone())
                                .expect("Failed to delete resources dir")
                        }
                    }

                    #[cfg(unix)]
                    {
                        std::os::unix::fs::symlink(resources_dir, local_resources)
                            .expect("Failed to sym link resource dir")
                    }

                    break;
                }
            } else {
                panic!("The resouce dir entered has not resources")
            }
        }
    }

    println!("cargo:rerun-if-changed=target");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.lock");
}

#[cfg(feature = "ultralight-resources")]
fn get_paths(possible_paths: &mut Vec<fs::DirEntry>, path_str: String) {
    let mut paths: Vec<DirEntry> = fs::read_dir(path_str)
        .expect("Could not read dir")
        .map(|f| f.unwrap())
        .filter(|file| file.path().to_string_lossy().contains("ul-next-sys"))
        .collect();
    // TODO: check if sort working
    paths.sort_by(|a, b| {
        a.metadata()
            .unwrap()
            .modified()
            .unwrap()
            .cmp(&b.metadata().unwrap().modified().unwrap())
    });
    possible_paths.append(&mut paths);
}
