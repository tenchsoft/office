/// A single Hanja candidate with its meaning.
#[derive(Debug, Clone)]
pub struct HanjaEntry {
    /// The Hanja character.
    pub hanja: &'static str,
    /// Korean meaning/reading.
    pub meaning: &'static str,
}
