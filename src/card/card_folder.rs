use strum::EnumIter;

/// The standard folders in a Deluge card.
#[derive(Debug, EnumIter)]
pub enum CardFolder {
    /// The `KITS` folder.
    Kits,
    /// The `SAMPLES` folder.
    Samples,
    /// The `SYNTHS` folder.
    Synths,
}

impl CardFolder {
    /// Get the name of the directory.
    pub const fn directory_name(&self) -> &'static str {
        match self {
            CardFolder::Kits => "KITS",
            CardFolder::Samples => "SAMPLES",
            CardFolder::Synths => "SYNTHS",
        }
    }
}
