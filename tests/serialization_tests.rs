//! This file contains a set of data driven tests.
//! I'm using <https://docs.rs/test-generator/latest/test_generator/> to generate tests using the
//! files present in //tests/data_tests
//!

#![cfg(test)]
extern crate test_generator;

use deluge::{deserialize_kit, deserialize_synth, serialize_kit, serialize_synth};
use pretty_assertions::assert_eq;
use test_generator::test_resources;

#[test_resources("tests/data_tests/KITS/*.XML")]
fn smoke_test_load_kit(resource: &str) {
    assert!(std::path::Path::new(resource).exists());

    let file_content = std::fs::read_to_string(resource).unwrap();
    let loading_result = deserialize_kit(&file_content);

    loading_result.unwrap();
}

#[test_resources("tests/data_tests/SYNTHS/*.XML")]
fn smoke_test_load_sound(resource: &str) {
    assert!(std::path::Path::new(resource).exists());

    let file_content = std::fs::read_to_string(resource).unwrap();
    let loading_result = deserialize_synth(&file_content);

    loading_result.unwrap();
}

#[test_resources("tests/data_tests/SYNTHS/*.XML")]
fn smoke_test_load_write_load_sound(resource: &str) {
    assert!(std::path::Path::new(resource).exists());

    let file_content = std::fs::read_to_string(resource).unwrap();
    let sound = deserialize_synth(&file_content).unwrap();
    let xml = serialize_synth(&sound).unwrap();
    let reloaded_sound = deserialize_synth(&xml).unwrap();

    assert_eq!(reloaded_sound, sound);
}

#[test_resources("tests/data_tests/KITS/*.XML")]
fn smoke_test_load_write_load_kit(resource: &str) {
    assert!(std::path::Path::new(resource).exists());

    let file_content = std::fs::read_to_string(resource).unwrap();
    let kit = deserialize_kit(&file_content).unwrap();
    let xml = serialize_kit(&kit).unwrap();
    let reloaded_kit = deserialize_kit(&xml).unwrap();

    assert_eq!(reloaded_kit, kit);
}

#[test_resources("tests/data_tests/COMMUNITY_PATCHES/SYNTHS/*.XML")]
fn smoke_test_load_write_load_sound_community_patches(resource: &str) {
    assert!(std::path::Path::new(resource).exists());

    let file_content = std::fs::read_to_string(resource).unwrap();
    let sound = deserialize_synth(&file_content).unwrap();
    let xml = serialize_synth(&sound).unwrap();
    let reloaded_sound = deserialize_synth(&xml).unwrap();

    assert_eq!(reloaded_sound, sound);
}

#[test_resources("tests/data_tests/COMMUNITY_PATCHES/KITS/*.XML")]
fn smoke_test_load_write_load_kit_community_patches(resource: &str) {
    assert!(std::path::Path::new(resource).exists());

    let file_content = std::fs::read_to_string(resource).unwrap();
    let kit = deserialize_kit(&file_content).unwrap();
    let xml = serialize_kit(&kit).unwrap();
    let reloaded_kit = deserialize_kit(&xml).unwrap();

    assert_eq!(reloaded_kit, kit);
}
