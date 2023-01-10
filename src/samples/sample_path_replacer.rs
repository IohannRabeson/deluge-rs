use crate::SamplePath;
use quick_xml::events::{BytesText, Event};
use quick_xml::{Reader, Writer};
use std::collections::BTreeMap;
use std::io::{BufRead, Write};

pub struct SamplePathReplacer {
    paths_to_replace: BTreeMap<SamplePath, SamplePath>,
}

impl SamplePathReplacer {
    pub fn new() -> Self {
        Self {
            paths_to_replace: BTreeMap::new(),
        }
    }

    pub fn add_path_replacement(&mut self, original: SamplePath, replacement: SamplePath) {
        self.paths_to_replace
            .insert(original, replacement);
    }

    pub fn rewrite<R, W>(&self, reader: R, writer: W) -> Result<(), quick_xml::Error>
    where
        R: BufRead,
        W: Write,
    {
        let mut reader = Reader::from_reader(reader);
        let mut writer = Writer::new(writer);
        let mut buffer: Vec<u8> = Vec::with_capacity(128);
        let mut is_in_filename_tag = false;

        while let Ok(mut event) = reader.read_event_into(&mut buffer) {
            match &event {
                Event::Start(tag_bytes) if tag_bytes.name().as_ref() == super::FILENAME_TAG => {
                    is_in_filename_tag = true;
                }
                Event::End(tag_bytes) if tag_bytes.name().as_ref() == super::FILENAME_TAG => {
                    is_in_filename_tag = false;
                }
                Event::Text(text_bytes) if is_in_filename_tag => {
                    if let Ok(text_utf8) = String::from_utf8(text_bytes.to_vec()) {
                        if let Ok(original_path) = SamplePath::new(text_utf8) {
                            if let Some(replacement_path) = self
                                .paths_to_replace
                                .get(&original_path)
                            {
                                event = Event::Text(BytesText::new(&replacement_path.to_string_lossy()).into_owned());
                            }
                        }
                    }
                }
                Event::Eof => break,
                _ => (),
            }

            writer.write_event(event)?;
            buffer.clear();
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::samples::sample_path_replacer::SamplePathReplacer;
    use crate::SamplePath;
    use nom::AsBytes;

    #[test]
    fn test_no_replacements() {
        use pretty_assertions::assert_eq;

        use std::io::Cursor;

        let file_content = include_bytes!("../data_tests/KITS/KIT030.XML");
        let transformer = SamplePathReplacer::new();
        let mut buffer = Vec::new();

        transformer
            .rewrite(file_content.as_bytes(), &mut buffer)
            .unwrap();

        let original_xml = xmltree::Element::parse(file_content.as_bytes()).unwrap();
        let transformed_xml = xmltree::Element::parse(Cursor::new(&buffer)).unwrap();

        assert_eq!(original_xml, transformed_xml);
    }

    #[test]
    fn test_replacements() {
        use crate::samples::read_sample_paths;
        use pretty_assertions::assert_eq;
        use std::io::Cursor;

        let file_content = include_bytes!("../data_tests/KITS/KIT030.XML");
        let mut transformer = SamplePathReplacer::new();
        let mut buffer = Vec::new();

        transformer.add_path_replacement(
            SamplePath::new("SAMPLES/ARTISTS/CHAZ/CB4-Cassette808_BD02.wav").unwrap(),
            SamplePath::new("SAMPLES/ARTISTS/CHAZ/CB4-Cassette808_BD02_YO.wav").unwrap(),
        );
        transformer.add_path_replacement(
            SamplePath::new("SAMPLES/ARTISTS/CHAZ/CB5-Cassette808_BD03.wav").unwrap(),
            SamplePath::new("SAMPLES/ARTISTS/CHAZ/CB5-Cassette808_BD03_YO.wav").unwrap(),
        );
        transformer.add_path_replacement(
            SamplePath::new("SAMPLES/ARTISTS/CHAZ/CB1-BD~1.WAV").unwrap(),
            SamplePath::new("SAMPLES/ARTISTS/CHAZ/CB1-BD~1_YO.WAV").unwrap(),
        );
        transformer.add_path_replacement(
            SamplePath::new("SAMPLES/ARTISTS/CHAZ/CB2-BD~1.WAV").unwrap(),
            SamplePath::new("SAMPLES/ARTISTS/CHAZ/CB2-BD~1_YO.WAV").unwrap(),
        );
        transformer.add_path_replacement(
            SamplePath::new("SAMPLES/ARTISTS/CHAZ/CB7-Cassette808_Rim_01.wav").unwrap(),
            SamplePath::new("SAMPLES/ARTISTS/CHAZ/CB7-Cassette808_Rim_01_YO.wav").unwrap(),
        );
        transformer.add_path_replacement(
            SamplePath::new("SAMPLES/ARTISTS/CHAZ/CB6-Cassette808_CP_01.wav").unwrap(),
            SamplePath::new("SAMPLES/ARTISTS/CHAZ/CB6-Cassette808_CP_01_YO.wav").unwrap(),
        );
        transformer.add_path_replacement(
            SamplePath::new("SAMPLES/ARTISTS/CHAZ/CB3-BELL.WAV").unwrap(),
            SamplePath::new("SAMPLES/ARTISTS/CHAZ/CB3-BELL_YO.WAV").unwrap(),
        );
        transformer.add_path_replacement(
            SamplePath::new("SAMPLES/ARTISTS/CHAZ/CB8-yo.wav").unwrap(),
            SamplePath::new("SAMPLES/ARTISTS/CHAZ/CB8-yo_YO.wav").unwrap(),
        );
        transformer
            .rewrite(file_content.as_bytes(), &mut buffer)
            .unwrap();

        let paths: Vec<SamplePath> = read_sample_paths(Cursor::new(&buffer)).collect();

        assert_eq!(8, paths.len());
        assert_eq!(
            SamplePath::new("SAMPLES/ARTISTS/CHAZ/CB4-Cassette808_BD02_YO.wav").unwrap(),
            paths[0]
        );
        assert_eq!(
            SamplePath::new("SAMPLES/ARTISTS/CHAZ/CB5-Cassette808_BD03_YO.wav").unwrap(),
            paths[1]
        );
        assert_eq!(SamplePath::new("SAMPLES/ARTISTS/CHAZ/CB1-BD~1_YO.WAV").unwrap(), paths[2]);
        assert_eq!(SamplePath::new("SAMPLES/ARTISTS/CHAZ/CB2-BD~1_YO.WAV").unwrap(), paths[3]);
        assert_eq!(
            SamplePath::new("SAMPLES/ARTISTS/CHAZ/CB7-Cassette808_Rim_01_YO.wav").unwrap(),
            paths[4]
        );
        assert_eq!(
            SamplePath::new("SAMPLES/ARTISTS/CHAZ/CB6-Cassette808_CP_01_YO.wav").unwrap(),
            paths[5]
        );
        assert_eq!(SamplePath::new("SAMPLES/ARTISTS/CHAZ/CB3-BELL_YO.WAV").unwrap(), paths[6]);
        assert_eq!(SamplePath::new("SAMPLES/ARTISTS/CHAZ/CB8-yo_YO.wav").unwrap(), paths[7]);
    }
}
