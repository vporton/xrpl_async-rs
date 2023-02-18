use derive_more::From;
use base58check::FromBase58CheckError;

struct FromXRPLengthError;

#[derive(From)]
enum FromXRPDecodingError {
    FromBase58Check(FromBase58CheckError),
    WrongPrefix,
    WrongLength(FromXRPLengthError),
}

// TODO: Unit test that `human_representation_starts_with` and `type_prefix` agree.
trait Encoding<const length: usize>: Sized {
    /// Byte added as prefix to sequence before encoding
    fn type_prefix() -> u8;
    /// The letter from which base58 representation starts
    fn human_representation_starts_with() -> char;
    fn bytes_without_prefix(&self) -> &[u8];
    fn from_bytes_without_prefix(bytes: &[u8]) -> Result<Self, FromXRPLengthError>;
    fn bytes(&self) -> Vec<u8> {
        type_prefix().once().chain(self.bytes_without_prefix()).collect()
    }
    fn encode(&self) -> String {
        bytes_without_prefix().as_bytes().to_base58check(Self::type_prefix())
    }
    fn decode(s: String) -> Result<Self, FromXRPDecodingError> {
        let (prefix, bytes) = s.from_base58check()?;
        if prefix != Self::type_prefix() {
            return Err(FromXRPDecodingError::WrongPrefix).into();
        }
        Self::from_bytes_without_prefix(bytes)
    }
}

/// Account address
struct Address([u8; 20]);

impl Encoding for Address {
    fn type_prefix() -> u8 {
        0x00
    }
    fn human_representation_starts_with() -> char {
        'r'
    }
    fn bytes_without_prefix(&self) -> &[u8] {
        self.0 as &[u8]
    }
    fn from_bytes_without_prefix(bytes: &[u8]) -> Result<Self, FromXRPLengthError> {
        Self(<[u8; 20]>::try_from(bytes)?)
        // Self(bytes.try_into::<[u8; 20]>()?)
    }
}

/// Account public key
struct AccountPublicKey([u8; 33]);

impl Encoding for AccountPublicKey {
    fn type_prefix() -> u8 {
        0x23
    }
    fn human_representation_starts_with() -> char {
        'a'
    }
}

/// Seed value (for secret keys)
struct SeedValue([u8; 16]);

impl Encoding for SeedValue {
    fn type_prefix() -> u8 {
        0x21
    }
    fn human_representation_starts_with() -> char {
        's'
    }
}

/// Validation public key or node public key
struct ValidationOrNodePublicKey([u8; 33]);

impl Encoding for ValidationOrNodePublicKey {
    fn type_prefix() -> u8 {
        0x1C
    }
    fn human_representation_starts_with() -> char {
        'n'
    }
}