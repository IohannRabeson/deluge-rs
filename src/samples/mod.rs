mod sample_path_replacer;

pub use sample_path_replacer::SamplePathReplacer;

use crate::SamplePath;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::io::BufRead;

/// Get the sample paths found in a patch.
/// This function does not check the XML really contains a Deluge patch.
pub fn read_sample_paths<'l>(reader: impl BufRead + 'l) -> impl Iterator<Item = SamplePath> + 'l {
    SamplesReader::new(reader)
}

struct SamplesReader<R: BufRead> {
    reader: Reader<R>,
    is_in_filename_tag: bool,
    buffer: Vec<u8>,
}

impl<R: BufRead> SamplesReader<R> {
    pub fn new(reader: R) -> Self {
        let mut reader = Reader::from_reader(reader);

        reader.trim_text(true);

        Self {
            reader,
            is_in_filename_tag: false,
            buffer: Vec::with_capacity(128),
        }
    }
}

const FILENAME_TAG: &[u8; 8] = b"fileName";

impl<R: BufRead> Iterator for SamplesReader<R> {
    type Item = SamplePath;

    fn next(&mut self) -> Option<Self::Item> {
        while let Ok(event) = self
            .reader
            .read_event_into(&mut self.buffer)
        {
            match event {
                Event::Start(tag_bytes) if tag_bytes.name().as_ref() == FILENAME_TAG => {
                    self.is_in_filename_tag = true;
                }
                Event::End(tag_bytes) if tag_bytes.name().as_ref() == FILENAME_TAG => {
                    self.is_in_filename_tag = false;
                }
                Event::Text(text_bytes) if self.is_in_filename_tag => {
                    if let Ok(text_utf8) = String::from_utf8(text_bytes.to_vec()) {
                        return SamplePath::new(&text_utf8).ok();
                    }
                }
                Event::Eof => break,
                _ => (),
            }

            self.buffer.clear();
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use crate::SamplePath;

    #[test]
    fn test_kit30() {
        use std::io::Cursor;

        let file_content = Cursor::new(include_str!("../data_tests/KITS/KIT030.XML"));
        let paths: Vec<SamplePath> = super::read_sample_paths(file_content).collect();

        assert_eq!(8, paths.len());
        assert_eq!(
            SamplePath::new("SAMPLES/ARTISTS/CHAZ/CB4-Cassette808_BD02.wav").unwrap(),
            paths[0]
        );
        assert_eq!(
            SamplePath::new("SAMPLES/ARTISTS/CHAZ/CB5-Cassette808_BD03.wav").unwrap(),
            paths[1]
        );
        assert_eq!(SamplePath::new("SAMPLES/ARTISTS/CHAZ/CB1-BD~1.WAV").unwrap(), paths[2]);
        assert_eq!(SamplePath::new("SAMPLES/ARTISTS/CHAZ/CB2-BD~1.WAV").unwrap(), paths[3]);
        assert_eq!(
            SamplePath::new("SAMPLES/ARTISTS/CHAZ/CB7-Cassette808_Rim_01.wav").unwrap(),
            paths[4]
        );
        assert_eq!(
            SamplePath::new("SAMPLES/ARTISTS/CHAZ/CB6-Cassette808_CP_01.wav").unwrap(),
            paths[5]
        );
        assert_eq!(SamplePath::new("SAMPLES/ARTISTS/CHAZ/CB3-BELL.WAV").unwrap(), paths[6]);
        assert_eq!(SamplePath::new("SAMPLES/ARTISTS/CHAZ/CB8-yo.wav").unwrap(), paths[7]);
    }
}
