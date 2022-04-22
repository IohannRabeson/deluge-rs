use std::path::{Path, PathBuf};
use test_case::test_case;

use crate::{values::SamplePath, PatchType};

use super::{filesystem::MockFileSystem, Card, CardError};

#[test]
fn test_check_root_directories_all_correct() {
    let fs = &mut MockFileSystem::default();

    fs.expect_get_directory_entries().returning(|path| {
        let mut paths: Vec<PathBuf> = Vec::new();

        paths.push(path.join("KITS"));
        paths.push(path.join("SAMPLES"));
        paths.push(path.join("SYNTHS"));

        Ok(paths)
    });

    assert_eq!(Ok(()), Card::check_root_directories(fs, &Path::new("big pullayo")));
}

#[test]
fn test_check_root_directories_first_missing() {
    let fs = &mut MockFileSystem::default();

    fs.expect_get_directory_entries().returning(|path| {
        let mut paths: Vec<PathBuf> = Vec::new();

        paths.push(path.join("PLITS"));
        paths.push(path.join("SAMPLES"));
        paths.push(path.join("SYNTHS"));

        Ok(paths)
    });

    assert_eq!(
        Err(CardError::MissingRootDirectory("KITS".into())),
        Card::check_root_directories(fs, &Path::new("big pullayo"))
    );
}

#[test]
fn test_check_root_directories_last_missing() {
    let fs = &mut MockFileSystem::default();

    fs.expect_get_directory_entries().returning(|path| {
        let mut paths: Vec<PathBuf> = Vec::new();

        paths.push(path.join("KITS"));
        paths.push(path.join("SAMPLES"));
        paths.push(path.join("FFYNYNTHS"));

        Ok(paths)
    });

    assert_eq!(
        Err(CardError::MissingRootDirectory("SYNTHS".into())),
        Card::check_root_directories(fs, &Path::new("big pullayo"))
    );
}

#[test]
fn test_open_card_non_existing_directory() {
    let fs = &mut MockFileSystem::default();

    fs.expect_directory_exists().times(1).return_const(false);
    fs.expect_get_directory_entries().times(0);
    let directory_path = Path::new("I_m_not_existings_duh");

    assert_eq!(
        Err(CardError::DirectoryDoesNotExists(directory_path.to_path_buf())),
        Card::open(fs, &directory_path)
    );
}

#[test]
fn test_open_card_ok() {
    let fs = &mut MockFileSystem::default();

    fs.expect_directory_exists().times(1).return_const(true);
    fs.expect_get_directory_entries().times(1).return_once(|path| {
        let mut paths: Vec<PathBuf> = Vec::new();

        paths.push(path.join("KITS"));
        paths.push(path.join("SAMPLES"));
        paths.push(path.join("SYNTHS"));

        Ok(paths)
    });

    assert!(Card::open(fs, &Path::new("I_m_existings")).is_ok());
}

fn create_valid_card(mut fs: MockFileSystem, root_directory: &'static Path) -> MockFileSystem {
    fs.expect_directory_exists().return_const(true);
    fs.expect_get_directory_entries()
        .with(mockall::predicate::eq(root_directory))
        .return_once(|path| {
            let mut paths: Vec<PathBuf> = Vec::new();

            paths.push(path.join("KITS"));
            paths.push(path.join("SAMPLES"));
            paths.push(path.join("SYNTHS"));

            Ok(paths)
        });

    fs
}

#[test_case("KIT000", Ok("KIT001") ; "KIT000")]
#[test_case("KIT", Ok("KIT000") ; "KIT")]
#[test_case("alariabiata", Ok("KIT000") ; "not default kit")]
#[test_case("KIT000A", Ok("KIT001") ; "KIT000A")]
#[test_case("KIT000Z", Ok("KIT001") ; "KIT000Z")]
#[test_case("KIT999", Err(CardError::NoMoreStandardName) ; "KIT999")]
fn test_get_next_patch_name(existing_patch_name: &str, expected_result: Result<&str, CardError>) {
    let root_directory = Path::new("I_exist");
    let mut fs = create_valid_card(MockFileSystem::default(), root_directory);
    let existing_patch_name_for_closure = existing_patch_name.to_string();
    fs.expect_get_directory_entries().return_once(|path| {
        let mut paths: Vec<PathBuf> = Vec::new();

        paths.push(path.join(existing_patch_name_for_closure));

        Ok(paths)
    });
    fs.expect_is_file().return_once(|_path| Ok(true));

    let card = Card::open(&fs, &Path::new("I_exist")).expect("open mocked card");
    let result = card.get_next_standard_patch_name(PatchType::Kit);

    assert_eq!(expected_result.map(|s| s.to_string()), result);
}

/// Check the next name always have a number greater than the
/// bigger number in the list of patches.
#[test]
fn test_get_next_patch_name_max() {
    let fs = &mut MockFileSystem::default();
    let root_directory = Path::new("I_exist");

    fs.expect_directory_exists().return_const(true);
    fs.expect_get_directory_entries()
        .with(mockall::predicate::eq(root_directory))
        .return_once(|path| {
            let mut paths: Vec<PathBuf> = Vec::new();

            paths.push(path.join("KITS"));
            paths.push(path.join("SAMPLES"));
            paths.push(path.join("SYNTHS"));

            Ok(paths)
        });

    fs.expect_get_directory_entries().return_once(|path| {
        let mut paths: Vec<PathBuf> = Vec::new();

        paths.push(path.join("KIT003"));
        paths.push(path.join("KIT007"));
        paths.push(path.join("KIT001"));

        Ok(paths)
    });
    fs.expect_is_file().return_const::<Result<bool, CardError>>(Ok(true));

    let card = Card::open(fs, &Path::new("I_exist")).expect("open mocked card");
    let patch_name = card.get_next_standard_patch_name(PatchType::Kit).unwrap();

    assert_eq!("KIT008", patch_name);
}

fn create_mocked_card<'l>(filesystem: &'l mut MockFileSystem, root_directory: &'static Path) -> Card<'l, MockFileSystem> {
    filesystem.expect_directory_exists().return_const(true);
    filesystem
        .expect_get_directory_entries()
        .with(mockall::predicate::eq(root_directory))
        .returning(|path| {
            let mut paths: Vec<PathBuf> = Vec::new();

            paths.push(path.join("KITS"));
            paths.push(path.join("SAMPLES"));
            paths.push(path.join("SYNTHS"));

            Ok(paths)
        });

    Card::open(filesystem, root_directory).expect("open mocked card")
}

#[test_case("root_dir/SAMPLES/A.WAV", Ok("SAMPLES/A.WAV"))]
#[test_case("OHLALA", Err(CardError::FileNotInCard(PathBuf::from("OHLALA"))))]
fn test_sample_path(input: &str, expected_result: Result<&str, CardError>) {
    let mut filesystem = MockFileSystem::new();
    let card = create_mocked_card(&mut filesystem, Path::new("root_dir"));
    let result = card.sample_path(Path::new(input));
    let expected_result = expected_result.map(|path| SamplePath::new(path).unwrap());

    assert_eq!(expected_result, result);
}
