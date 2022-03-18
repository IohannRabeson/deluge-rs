#![cfg(test)]
extern crate test_generator;

use test_generator::test_resources;

use deluge::{load_kit, load_sound, save_kit, save_sound};

#[test_resources("tests/data_tests/KITS/*.XML")]
fn smoke_test_load_kit(resource: &str) {
    assert!(std::path::Path::new(resource).exists());

    let file_content = std::fs::read_to_string(resource).unwrap();
    let loading_result = load_kit(&file_content);

    loading_result.unwrap();
}

#[test_resources("tests/data_tests/SYNTHS/*.XML")]
fn smoke_test_load_sound(resource: &str) {
    assert!(std::path::Path::new(resource).exists());

    let file_content = std::fs::read_to_string(resource).unwrap();
    let loading_result = load_sound(&file_content);

    loading_result.unwrap();
}

use pretty_assertions::assert_eq;

#[test_resources("tests/data_tests/SYNTHS/*.XML")]
fn smoke_test_load_write_load_sound(resource: &str) {
    assert!(std::path::Path::new(resource).exists());

    let file_content = std::fs::read_to_string(resource).unwrap();
    let sound = load_sound(&file_content).unwrap();
    let xml = save_sound(&sound).unwrap();
    let reloaded_sound = load_sound(&xml).unwrap();

    assert_eq!(reloaded_sound, sound);
}

#[test_resources("tests/data_tests/KITS/*.XML")]
fn smoke_test_load_write_load_kit(resource: &str) {
    assert!(std::path::Path::new(resource).exists());

    let file_content = std::fs::read_to_string(resource).unwrap();
    let kit = load_kit(&file_content).unwrap();
    let xml = save_kit(&kit).unwrap();
    let reloaded_kit = load_kit(&xml).unwrap();

    assert_eq!(reloaded_kit, kit);
}

#[test_resources("tests/data_tests/COMMUNITY_PATCHES/SYNTHS/*.XML")]
fn smoke_test_load_write_load_sound_community_patches(resource: &str) {
    assert!(std::path::Path::new(resource).exists());

    let file_content = std::fs::read_to_string(resource).unwrap();
    let sound = load_sound(&file_content).unwrap();
    let xml = save_sound(&sound).unwrap();
    let reloaded_sound = load_sound(&xml).unwrap();

    assert_eq!(reloaded_sound, sound);
}

#[test_resources("tests/data_tests/COMMUNITY_PATCHES/KITS/*.XML")]
fn smoke_test_load_write_load_kit_community_patches(resource: &str) {
    assert!(std::path::Path::new(resource).exists());

    let file_content = std::fs::read_to_string(resource).unwrap();
    let kit = load_kit(&file_content).unwrap();
    let xml = save_kit(&kit).unwrap();
    let reloaded_kit = load_kit(&xml).unwrap();

    assert_eq!(reloaded_kit, kit);
}
