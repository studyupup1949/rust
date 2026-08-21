// TODO: Maybe remove pub
pub use bip39::{Mnemonic, MnemonicType, Language, Seed};
pub use zeroize::Zeroize;


pub struct BIP39API;

/// # Secret Phrase
/// 
/// A **Secret Mnemonic Phrase** used to generate the seed for accounts of length 18 words using BIP39.
#[derive(Debug,Clone,Zeroize)]
#[zeroize(drop)]
pub struct SecretPhrase(String);

impl BIP39API {
    /// # Generate Mnemonic Phrase
    /// 
    /// This function is a simple to use function to generate a Mnemonic Phrase from cryptographic randomness.
    /// 
    /// It accepts one input:
    /// 
    /// - Language
    /// 
    /// ## Language
    /// 
    /// Default: `Language::English`
    /// 
    /// This is the language of the phrases. These are the following options:
    /// 
    /// - English
    /// - ChineseSimplified
    /// - ChineseTraditional
    /// - French
    /// - Italian
    /// - Japanese
    /// - Korean
    /// - Spanish
    /// 
    pub fn generate(language: bip39::Language) -> SecretPhrase {
        // Generate Mnemonic Phrase of 18 words
        let mnemonic = Mnemonic::new(MnemonicType::Words18, language);
        let phrase: &str = mnemonic.phrase();

        // Make sure there are 18 words
        assert_eq!(phrase.split(" ").count(), 18);

        return SecretPhrase(phrase.to_string())
    }
}

impl SecretPhrase {
    /// Create `SecretPhrase` from String or &str
    pub fn new<T: AsRef<str>>(phrase: T) -> Self {
        Self(phrase.as_ref().to_string())
    }
    /// Derive Seed From Mnemonic Phrase using password and making sure the language is correct
    /// 
    /// The seed is **64 bytes**.
    pub fn derive_seed(&self, password: &str, language: bip39::Language) -> bip39::Seed {
        let mnenomic = Mnemonic::from_phrase(&self.0, language).expect("[ERROR] Failed To Convert From Mnemonic Phrase To Seed");
        let seed = Seed::new(&mnenomic,password);
        return seed
    }
    /// Validate Mnemonic Phrase
    pub fn validate(&self, language: bip39::Language) -> bool {
        let is_valid = Mnemonic::validate(&self.0,language).is_ok();
        
        return is_valid
    }
    /// Zeroize Secret Phrase
    pub fn clear(mut self) -> () {
        self.zeroize()
    }
    /// Export Secret Phrase to &str
    pub fn export_phrase(&self) -> &str {
        return &self.0
    }
}

#[test]
fn test_mnemonic_phrase(){
    // Generate Mnemnonic Phrase
    let phrase = BIP39API::generate(Language::English);
    // Derive Seed From Mnemonic Phrase
    let seed = phrase.derive_seed("Password1234", Language::English);

    // Get Final Seed (64 bytes)
    let final_seed = seed.as_bytes();
}