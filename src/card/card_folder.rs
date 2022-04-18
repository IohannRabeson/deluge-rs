use strum::EnumIter;

#[derive(Debug, EnumIter)]
pub enum CardFolder {
    Kits,
    Samples,
    Synths,
}

impl CardFolder {
    pub const fn directory_name(&self) -> &'static str {
        match self {
            CardFolder::Kits => "KITS",
            CardFolder::Samples => "SAMPLES",
            CardFolder::Synths => "SYNTHS",
        }
    }
}
