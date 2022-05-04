use crate::Sound;

/// Default implementation for Kit
///
/// The default Synth is exactly like the Deluge would create it for a default synth patch without any user changes.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Synth {
    pub sound: Sound,
}

#[cfg(test)]
mod tests {
    use crate::{deserialize_synth, Synth};
    use pretty_assertions::assert_eq;

    #[test]
    fn default_synth_test() {
        let default_synth = Synth::default();
        let expected_default_synth = deserialize_synth(include_str!("data_tests/default/SYNTh Default.XML")).unwrap();

        assert_eq!(expected_default_synth, default_synth)
    }
}
