// Descriptor wallet library extending bitcoin & miniscript functionality
// by LNP/BP Association (https://lnp-bp.org)
// Written in 2020-2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the Apache-2.0 License
// along with this software.
// If not, see <https://opensource.org/licenses/Apache-2.0>.

use std::convert::TryFrom;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use amplify::Wrapper;
use bitcoin::blockdata::opcodes;
use bitcoin::blockdata::opcodes::All;
use bitcoin::blockdata::script::*;
use bitcoin::hashes::hex::ToHex;
use bitcoin::hashes::Hash;
use bitcoin::{
    secp256k1, Address, Network, PubkeyHash, ScriptHash, WPubkeyHash,
    WScriptHash,
};
use miniscript::ToPublicKey;

use crate::Category;

/// Script whose knowledge is required for spending some specific transaction
/// output. This is the deepest nested version of Bitcoin script containing no
/// hashes of other scripts, including P2SH redeemScript hashes or
/// witnessProgram (hash or witness script), or public key hashes
#[derive(
    Wrapper, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug,
    Display, From
)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", transparent)
)]
#[display("{0}", alt = "{_0:x}")]
#[wrapper(LowerHex, UpperHex)]
pub struct LockScript(Script);

impl strict_encoding::Strategy for LockScript {
    type Strategy = strict_encoding::strategies::Wrapped;
}

/// A representation of `scriptPubkey` data used during SegWit signing procedure
#[derive(
    Wrapper, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug,
    Display, From
)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", transparent)
)]
#[display("{0}", alt = "{_0:x}")]
#[wrapper(LowerHex, UpperHex)]
pub struct ScriptCode(Script);

/// A content of `scriptPubkey` from a transaction output
#[derive(
    Wrapper, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug,
    Display, From
)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", transparent)
)]
#[display("{0}", alt = "{_0:x}")]
#[wrapper(LowerHex, UpperHex)]
pub struct PubkeyScript(Script);

impl strict_encoding::Strategy for PubkeyScript {
    type Strategy = strict_encoding::strategies::Wrapped;
}

impl PubkeyScript {
    pub fn address(&self, network: Network) -> Option<Address> {
        Address::from_script(self.as_inner(), network)
    }

    pub fn script_code(&self) -> ScriptCode {
        if self.0.is_v0_p2wpkh() {
            let pubkey_hash = PubkeyHash::from_slice(&self.0[2..22])
                .expect("PubkeyHash hash length failure");
            ScriptCode::from_inner(Script::new_p2pkh(&pubkey_hash))
        } else {
            ScriptCode::from_inner(self.to_inner())
        }
    }
}

impl From<WPubkeyHash> for PubkeyScript {
    fn from(wpkh: WPubkeyHash) -> Self {
        Script::new_v0_wpkh(&wpkh).into()
    }
}

/// A content of `sigScript` from a transaction input
#[derive(
    Wrapper, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug,
    Display, From
)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", transparent)
)]
#[display("{0}", alt = "{_0:x}")]
#[wrapper(LowerHex, UpperHex)]
pub struct SigScript(Script);

impl strict_encoding::Strategy for SigScript {
    type Strategy = strict_encoding::strategies::Wrapped;
}

/// A content of the `witness` field from a transaction input according to
/// BIP-141
#[derive(
    Wrapper, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug, From
)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", transparent)
)]
pub struct Witness(Vec<Vec<u8>>);

impl strict_encoding::Strategy for Witness {
    type Strategy = strict_encoding::strategies::Wrapped;
}

impl Display for Witness {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("[\n")?;
        for vec in self.as_inner().iter() {
            writeln!(f, "{}", vec.to_hex())?;
        }
        f.write_str("]\n")
    }
}

/// `redeemScript` as part of the `witness` or `sigScript` structure; it is
///  hashed for P2(W)SH output
#[derive(
    Wrapper, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug,
    Display, From
)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", transparent)
)]
#[display("{0}", alt = "{_0:x}")]
#[wrapper(LowerHex, UpperHex)]
pub struct RedeemScript(Script);

impl strict_encoding::Strategy for RedeemScript {
    type Strategy = strict_encoding::strategies::Wrapped;
}

impl RedeemScript {
    pub fn script_hash(&self) -> ScriptHash {
        self.as_inner().script_hash()
    }
    pub fn to_p2sh(&self) -> PubkeyScript {
        self.to_pubkey_script(Category::Hashed)
    }
}

impl ToPubkeyScript for RedeemScript {
    fn to_pubkey_script(&self, strategy: Category) -> PubkeyScript {
        LockScript::from(self.clone()).to_pubkey_script(strategy)
    }
}

/// A content of the script from `witness` structure; en equivalent of
/// `redeemScript` for witness-based transaction inputs. However, unlike
/// [`RedeemScript`], [`WitnessScript`] produce SHA256-based hashes of
/// [`WScriptHash`] type
#[derive(
    Wrapper, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug,
    Display, From
)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", transparent)
)]
#[display("{0}", alt = "{_0:x}")]
#[wrapper(LowerHex, UpperHex)]
pub struct WitnessScript(Script);

impl strict_encoding::Strategy for WitnessScript {
    type Strategy = strict_encoding::strategies::Wrapped;
}

impl WitnessScript {
    pub fn script_hash(&self) -> WScriptHash {
        self.as_inner().wscript_hash()
    }
    pub fn to_p2wsh(&self) -> PubkeyScript {
        self.to_pubkey_script(Category::SegWit)
    }
    pub fn to_p2sh_wsh(&self) -> PubkeyScript {
        self.to_pubkey_script(Category::Nested)
    }
}

impl ToPubkeyScript for WitnessScript {
    fn to_pubkey_script(&self, strategy: Category) -> PubkeyScript {
        LockScript::from(self.clone()).to_pubkey_script(strategy)
    }
}

impl From<LockScript> for WitnessScript {
    fn from(lock_script: LockScript) -> Self {
        WitnessScript(lock_script.to_inner())
    }
}

impl From<LockScript> for RedeemScript {
    fn from(lock_script: LockScript) -> Self {
        RedeemScript(lock_script.to_inner())
    }
}

impl From<WitnessScript> for LockScript {
    fn from(witness_script: WitnessScript) -> Self {
        LockScript(witness_script.to_inner())
    }
}

impl From<RedeemScript> for LockScript {
    fn from(redeem_script: RedeemScript) -> Self {
        LockScript(redeem_script.to_inner())
    }
}

/// Any valid branch of Tapscript (BIP-342)
#[derive(
    Wrapper, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug,
    Display, From
)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", transparent)
)]
#[display("{0}", alt = "{_0:x}")]
#[wrapper(LowerHex, UpperHex)]
pub struct TapScript(Script);

// TODO #15: Add address generation impl once Taproot will be out

impl strict_encoding::Strategy for TapScript {
    type Strategy = strict_encoding::strategies::Wrapped;
}

/// Version of the WitnessProgram: first byte of `scriptPubkey` in
/// transaction output for transactions starting with opcodes ranging from 0
/// to 16 (inclusive).
///
/// Structure helps to limit possible version of the witness according to the
/// specification; if a plain `u8` type will be used instead it will mean that
/// version > 16, which is incorrect.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate")
)]
#[repr(u8)]
pub enum WitnessVersion {
    /// Current, initial version of Witness Program. Used for P2WPKH and P2WPK
    /// outputs
    #[display("v0")]
    V0 = 0,

    /// Forthcoming second version of Witness Program, which (most probably)
    /// will be used for Taproot
    #[display("v1")]
    V1 = 1,

    /// Future (unsupported) version of Witness Program
    #[display("v2")]
    V2 = 2,

    /// Future (unsupported) version of Witness Program
    #[display("v3")]
    V3 = 3,

    /// Future (unsupported) version of Witness Program
    #[display("v4")]
    V4 = 4,

    /// Future (unsupported) version of Witness Program
    #[display("v5")]
    V5 = 5,

    /// Future (unsupported) version of Witness Program
    #[display("v6")]
    V6 = 6,

    /// Future (unsupported) version of Witness Program
    #[display("v7")]
    V7 = 7,

    /// Future (unsupported) version of Witness Program
    #[display("v8")]
    V8 = 8,

    /// Future (unsupported) version of Witness Program
    #[display("v9")]
    V9 = 9,

    /// Future (unsupported) version of Witness Program
    #[display("v10")]
    V10 = 10,

    /// Future (unsupported) version of Witness Program
    #[display("v11")]
    V11 = 11,

    /// Future (unsupported) version of Witness Program
    #[display("v12")]
    V12 = 12,

    /// Future (unsupported) version of Witness Program
    #[display("v13")]
    V13 = 13,

    /// Future (unsupported) version of Witness Program
    #[display("v14")]
    V14 = 14,

    /// Future (unsupported) version of Witness Program
    #[display("v15")]
    V15 = 15,

    /// Future (unsupported) version of Witness Program
    #[display("v16")]
    V16 = 16,
}

/// A error covering only one possible failure in WitnessVersion creation:
/// when the provided version > 16
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display, Error
)]
#[display(doc_comments)]
pub enum WitnessVersionError {
    /// The opocde provided for the version construction is incorrect
    IncorrectOpcode,

    /// Incorrect witness version string representation
    IncorrectString,
}

impl FromStr for WitnessVersion {
    type Err = WitnessVersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.to_lowercase().starts_with('v') {
            return Err(WitnessVersionError::IncorrectString);
        }
        WitnessVersion::try_from(
            s[1..]
                .parse::<u8>()
                .map_err(|_| WitnessVersionError::IncorrectString)?,
        )
    }
}

impl TryFrom<u8> for WitnessVersion {
    type Error = WitnessVersionError;

    /// Takes bitcoin Script value and returns either corresponding version of
    /// the Witness program (for opcodes in range of `OP_0`..`OP_16`) or
    /// [WitnessVersionError::IncorrectOpcode] error for the rest of opcodes
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use WitnessVersion::*;
        Ok(match value {
            0 => V0,
            1 => V1,
            2 => V2,
            3 => V3,
            4 => V4,
            5 => V5,
            6 => V6,
            7 => V7,
            8 => V8,
            9 => V9,
            10 => V10,
            11 => V11,
            12 => V12,
            13 => V13,
            14 => V14,
            15 => V15,
            16 => V16,
            _ => return Err(WitnessVersionError::IncorrectOpcode),
        })
    }
}

impl TryFrom<opcodes::All> for WitnessVersion {
    type Error = WitnessVersionError;

    /// Takes bitcoin Script opcode and returns either corresponding version of
    /// the Witness program (for opcodes in range of `OP_0`..`OP_16`) or
    /// [WitnessVersionError::IncorrectOpcode] error for the rest of opcodes
    fn try_from(value: All) -> Result<Self, Self::Error> {
        WitnessVersion::try_from(value.into_u8())
    }
}

impl<'a> TryFrom<Instruction<'a>> for WitnessVersion {
    type Error = WitnessVersionError;

    /// Takes bitcoin Script instruction (parsed opcode) and returns either
    /// corresponding version of the Witness program (for push-num instructions)
    /// or [WitnessVersionError::IncorrectOpcode] error for the rest of opcodes
    fn try_from(instruction: Instruction<'a>) -> Result<Self, Self::Error> {
        match instruction {
            Instruction::<'a>::Op(op) => Self::try_from(op),
            _ => Err(WitnessVersionError::IncorrectOpcode),
        }
    }
}

impl From<WitnessVersion> for opcodes::All {
    /// Converts `WitnessVersion` instance into corresponding Bitcoin script
    /// opcode (`OP_0`..`OP_16`)
    fn from(ver: WitnessVersion) -> Self {
        opcodes::All::from(ver as u8)
    }
}

#[derive(
    Wrapper, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug, From
)]
pub struct WitnessProgram(Vec<u8>);

impl strict_encoding::Strategy for WitnessProgram {
    type Strategy = strict_encoding::strategies::Wrapped;
}

impl Display for WitnessProgram {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.as_inner().to_hex())
    }
}

impl From<WPubkeyHash> for WitnessProgram {
    fn from(wpkh: WPubkeyHash) -> Self {
        WitnessProgram(wpkh.to_vec())
    }
}

impl From<WScriptHash> for WitnessProgram {
    fn from(wsh: WScriptHash) -> Self {
        WitnessProgram(wsh.to_vec())
    }
}

/// Scripting data for both transaction output and spending transaction input
/// parts that can be generated from some complete bitcoin Script
/// ([`LockScript`]) or public key using particular [`Category`]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Default)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate")
)]
pub struct ScriptSet {
    pub pubkey_script: PubkeyScript,
    pub sig_script: SigScript,
    pub witness_script: Option<Witness>,
}

impl Display for ScriptSet {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.sig_script,
            self.witness_script
                .as_ref()
                .map(Witness::to_string)
                .unwrap_or_default(),
            self.pubkey_script
        )
    }
}

impl ScriptSet {
    /// Detects whether the structure contains witness data
    #[inline]
    pub fn has_witness(&self) -> bool {
        self.witness_script != None
    }

    /// Detects whether the structure is either P2SH-P2WPKH or P2SH-P2WSH
    pub fn is_witness_sh(&self) -> bool {
        return !self.sig_script.as_inner().is_empty() && self.has_witness();
    }

    /// Tries to convert witness-based script structure into pre-SegWit – and
    /// vice verse. Returns `true` if the conversion is possible and was
    /// successful, `false` if the conversion is impossible; in the later case
    /// the `self` is not changed. The conversion is impossible in the following
    /// cases:
    /// * for P2SH-P2WPKH or P2SH-P2WPSH variants (can be detected with
    ///   [ScriptSet::is_witness_sh] function)
    /// * for scripts that are internally inconsistent
    pub fn transmutate(&mut self, use_witness: bool) -> bool {
        // We can't transmutate P2SH-contained P2WSH/P2WPKH
        if self.is_witness_sh() {
            return false;
        }
        if self.has_witness() != use_witness {
            if use_witness {
                self.witness_script = Some(
                    self.sig_script
                        .as_inner()
                        .instructions_minimal()
                        .filter_map(|instr| {
                            if let Ok(Instruction::PushBytes(bytes)) = instr {
                                Some(bytes.to_vec())
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<Vec<u8>>>()
                        .into(),
                );
                self.sig_script = SigScript::default();
                true
            } else if let Some(ref witness_script) = self.witness_script {
                self.sig_script = witness_script
                    .as_inner()
                    .iter()
                    .fold(Builder::new(), |builder, bytes| {
                        builder.push_slice(bytes)
                    })
                    .into_script()
                    .into();
                self.witness_script = None;
                true
            } else {
                false
            }
        } else {
            false
        }
    }
}

/// Conversion to [`LockScript`], which later may be used for creating different
/// end-point scripts, like [`PubkeyScript`], [`SigScript`], [`Witness`]
/// etc.
pub trait ToLockScript {
    fn to_lock_script(&self, strategy: Category) -> LockScript;
}

/// Conversion for data types (public keys, different types of script) into
/// a `pubkeyScript` (using [`PubkeyScript`] type) using particular conversion
/// [`Category`]
pub trait ToPubkeyScript {
    fn to_pubkey_script(&self, strategy: Category) -> PubkeyScript;
}

/// Script set generation from public keys or a given [`LockScript`] (with
/// [`TapScript`] support planned for the future).
pub trait ToScripts
where
    Self: ToPubkeyScript,
{
    fn to_scripts(&self, strategy: Category) -> ScriptSet {
        ScriptSet {
            pubkey_script: self.to_pubkey_script(strategy),
            sig_script: self.to_sig_script(strategy),
            witness_script: self.to_witness(strategy),
        }
    }
    fn to_sig_script(&self, strategy: Category) -> SigScript;
    fn to_witness(&self, strategy: Category) -> Option<Witness>;
}

impl ToPubkeyScript for LockScript {
    fn to_pubkey_script(&self, strategy: Category) -> PubkeyScript {
        match strategy {
            Category::Bare => self.to_inner().into(),
            Category::Hashed => Script::new_p2sh(&self.script_hash()).into(),
            Category::SegWit => Script::new_v0_wsh(&self.wscript_hash()).into(),
            Category::Nested => {
                // Here we support only V0 version, since V1 version can't
                // be generated from `LockScript` and will require
                // `TapScript` source
                let redeem_script = LockScript::from(
                    self.to_pubkey_script(Category::SegWit).to_inner(),
                );
                Script::new_p2sh(&redeem_script.script_hash()).into()
            }
            Category::Taproot => unimplemented!(),
        }
    }
}

impl ToScripts for LockScript {
    fn to_sig_script(&self, strategy: Category) -> SigScript {
        match strategy {
            // sigScript must contain just a plain signatures, which will be
            // added later
            Category::Bare => SigScript::default(),
            Category::Hashed => Builder::new()
                .push_slice(WitnessScript::from(self.clone()).as_bytes())
                .into_script()
                .into(),
            Category::Nested => {
                // Here we support only V0 version, since V1 version can't
                // be generated from `LockScript` and will require
                // `TapScript` source
                let redeem_script = LockScript::from(
                    self.to_pubkey_script(Category::SegWit).to_inner(),
                );
                Builder::new()
                    .push_slice(redeem_script.as_bytes())
                    .into_script()
                    .into()
            }
            // For any segwit version the sigScript must be empty (with the
            // exception to the case of P2SH-embedded outputs, which is already
            // covered above
            _ => SigScript::default(),
        }
    }

    fn to_witness(&self, strategy: Category) -> Option<Witness> {
        match strategy {
            Category::Bare | Category::Hashed => None,
            Category::SegWit | Category::Nested => {
                let witness_script = WitnessScript::from(self.clone());
                Some(Witness::from_inner(vec![witness_script.to_bytes()]))
            }
            Category::Taproot => unimplemented!(),
        }
    }
}

impl ToLockScript for bitcoin::PublicKey {
    fn to_lock_script(&self, strategy: Category) -> LockScript {
        match strategy {
            Category::Bare => Script::new_p2pk(self).into(),
            Category::Hashed => Script::new_p2pkh(&self.pubkey_hash()).into(),
            // TODO #16: Detect uncompressed public key and return error
            Category::SegWit => Script::new_v0_wpkh(
                &self
                    .wpubkey_hash()
                    .expect("Uncompressed public key used in witness script"),
            )
            .into(),
            Category::Nested => {
                let redeem_script = self.to_pubkey_script(Category::SegWit);
                Script::new_p2sh(&redeem_script.script_hash()).into()
            }
            Category::Taproot => unimplemented!(),
        }
    }
}

impl ToPubkeyScript for bitcoin::PublicKey {
    fn to_pubkey_script(&self, strategy: Category) -> PubkeyScript {
        self.to_lock_script(strategy).into_inner().into()
    }
}

impl ToScripts for bitcoin::PublicKey {
    fn to_sig_script(&self, strategy: Category) -> SigScript {
        match strategy {
            // sigScript must contain just a plain signatures, which will be
            // added later
            Category::Bare => SigScript::default(),
            Category::Hashed => Builder::new()
                .push_slice(&self.to_bytes())
                .into_script()
                .into(),
            Category::Nested => {
                let redeem_script = LockScript::from(
                    self.to_pubkey_script(Category::SegWit).into_inner(),
                );
                Builder::new()
                    .push_slice(redeem_script.as_bytes())
                    .into_script()
                    .into()
            }
            // For any segwit version the sigScript must be empty (with the
            // exception to the case of P2SH-embedded outputs, which is already
            // covered above
            _ => SigScript::default(),
        }
    }

    fn to_witness(&self, strategy: Category) -> Option<Witness> {
        match strategy {
            Category::Bare | Category::Hashed => None,
            Category::SegWit | Category::Nested => {
                Some(Witness::from_inner(vec![self.to_bytes()]))
            }
            Category::Taproot => unimplemented!(),
        }
    }
}

impl ToLockScript for secp256k1::PublicKey {
    #[inline]
    fn to_lock_script(&self, strategy: Category) -> LockScript {
        bitcoin::PublicKey {
            compressed: true,
            key: *self,
        }
        .to_lock_script(strategy)
    }
}

impl ToPubkeyScript for secp256k1::PublicKey {
    fn to_pubkey_script(&self, strategy: Category) -> PubkeyScript {
        self.to_lock_script(strategy).into_inner().into()
    }
}

impl ToScripts for secp256k1::PublicKey {
    #[inline]
    fn to_sig_script(&self, strategy: Category) -> SigScript {
        bitcoin::PublicKey {
            compressed: true,
            key: *self,
        }
        .to_sig_script(strategy)
    }

    #[inline]
    fn to_witness(&self, strategy: Category) -> Option<Witness> {
        bitcoin::PublicKey {
            compressed: true,
            key: *self,
        }
        .to_witness(strategy)
    }
}

pub trait ToP2pkh {
    fn to_p2pkh(&self) -> PubkeyScript;
    fn to_p2wpkh(&self) -> PubkeyScript;
    fn to_p2sh_wpkh(&self) -> PubkeyScript;
}

impl<T> ToP2pkh for T
where
    T: ToPublicKey,
{
    fn to_p2pkh(&self) -> PubkeyScript {
        self.to_public_key().to_pubkey_script(Category::Hashed)
    }

    fn to_p2wpkh(&self) -> PubkeyScript {
        self.to_public_key().to_pubkey_script(Category::SegWit)
    }

    fn to_p2sh_wpkh(&self) -> PubkeyScript {
        self.to_public_key().to_pubkey_script(Category::Nested)
    }
}
