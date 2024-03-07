//! StegCloak
//!
//! Hides secrets inside test by compressing and encrypting the secret before cloaking it
//! with special unicode invisible characters. It can be used to safely watermark strings,
//! invisible scripts on webpages, texts on social media, or for any other covert communication.
//! Completely invisible!
//!
//! Inspired by the original javascript [stegcloak](https://github.com/KuroLabs/stegcloak).
//!
//! This is imcompatible with the original js stegcloak. However, this compiles to wasm, so it can
//! also be used in javascript.
//!
//! # Warning
//!
//! This is currently under dev. Algorithm may be changed at any time, and previously encoded
//! messages may no longer be compatible with the new version.
//!
//! Every effort has been made to be cryptigraphically secure, however, this _should not_ be
//! relied on for any sensitive or secure communications! Author absolves self from all possible
//! issues that could arise from usage of this software.
//!
//! StegCloak doesn't solve the Alice-Bob-Warden problem, it's powerful only when people are not
//! looking for it and it helps you achieve that really well, given its invisible properties around
//! the web! It could be safely used for watermarking in forums, invisible tweets, social media etc.
//! Please don't use it when you know there's someone who is actively sniffing your data - looking at
//! the unicode characters through a data analysis tool. In that case, even though the secret encoded
//! cannot be deciphered, the fact lies that the Warden (middle-man) knows some secret communication
//! took place, because he would have noticed an unusual amount of special invisible characters.
//!
//! THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT
//! LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
//! IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY,
//! WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE
//! OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

mod codec;
mod compact;
mod crypto;

use codec::CodecError;
use compact::DeCompressError;
use crypto::DeEncryptError;

pub mod encrypt {
    use super::StegError;

    /// Hide an encrypted secret inside a message
    ///
    /// # Arguments
    ///
    /// * `secret` - The secret you want to hide
    /// * `password` - The password to encrypt the secret with
    /// * `integrity` - Create message that protects against tampering
    /// * `message` - The visible text everybody else will see
    ///
    /// # Examples
    ///
    /// ```rust
    ///     stegcloak::encrypt::hide("mysecret", "mypassword", false, "cover text"); // -> "cover text"
    /// ```
    ///
    pub fn hide(
        secret: impl AsRef<str>,
        password: impl AsRef<str>,
        integrity: bool,
        message: impl AsRef<str>,
    ) -> Result<String, StegError> {
        let secret = secret.as_ref();
        let password = password.as_ref();
        let message = message.as_ref();

        super::_hide(true, integrity, secret, password, message)
    }

    /// Reveal an encrypted secret inside a message
    ///
    /// # Arguments
    ///
    /// * `password` - The password to decrypt the secret with
    /// * `integrity` - Verify integrity; this must be `true` if the message was created with integrity option.
    ///                 This must be `false` if message was created without integrity option
    /// * `message` - The visible text everybody else sees
    ///
    /// # Examples
    /// ```rust
    ///     stegcloak::encrypt::reveal("mypassword", false, "cover text"); // -> "mysecret"
    /// ```
    ///
    pub fn reveal(
        password: impl AsRef<str>,
        integrity: bool,
        message: impl AsRef<str>,
    ) -> Result<String, StegError> {
        let password = password.as_ref();
        let message = message.as_ref();

        super::_reveal(true, integrity, password, message)
    }
}

pub mod plaintext {
    use super::StegError;

    /// Hide a plaintext secret inside a message
    ///
    /// Warn: The secret will be in plaintext! Anyone can freely decode it!
    ///
    /// # Arguments
    ///
    /// * `secret` - The secret you want to hide
    /// * `message` - The visible text everybody else will see
    ///
    /// # Examples
    ///
    /// ```rust
    ///     stegcloak::plaintext::hide("mysecret", "cover text"); // -> "cover text"
    /// ```
    ///
    pub fn hide(secret: impl AsRef<str>, message: impl AsRef<str>) -> Result<String, StegError> {
        let secret = secret.as_ref();
        let message = message.as_ref();

        super::_hide(false, false, secret, "", message)
    }

    /// Reveal a plaintext secret inside a message
    ///
    /// # Arguments
    ///
    /// * `message` - The visible text everybody else sees
    ///
    /// # Examples
    ///
    /// ```rust
    ///     stegcloak::plaintext::reveal("cover text"); // -> "mysecret"
    /// ```
    ///
    pub fn reveal(message: impl AsRef<str>) -> Result<String, StegError> {
        let message = message.as_ref();

        super::_reveal(false, false, "", message)
    }
}

fn _hide(
    encrypt: bool,
    integrity: bool,
    secret: &str,
    password: &str,
    message: &str,
) -> Result<String, StegError> {
    // minimum 1 space required
    let Some(space_pos) = message.find(' ') else {
        return Err(StegError::SpaceRequired);
    };

    let secret = compact::compress(secret)?;
    let data = if encrypt {
        crypto::encrypt(password, &secret, integrity)?
    } else {
        secret
    };

    let encoded = codec::encode(&data);

    let mut message = message.to_owned();
    message.insert_str(space_pos + 1, &encoded);

    Ok(message)
}

fn _reveal(
    encrypt: bool,
    integrity: bool,
    password: &str,
    message: &str,
) -> Result<String, StegError> {
    if !message.contains(' ') {
        return Err(StegError::SpaceRequired);
    }

    let decoded = codec::decode(message)?;
    let data = if encrypt {
        crypto::decrypt(password, &decoded, integrity)?
    } else {
        decoded
    };

    Ok(compact::decompress(&data)?)
}

#[derive(Debug, thiserror::Error)]
pub enum StegError {
    #[error("Text does not contain a space")]
    SpaceRequired,
    #[error("Failed Compression/Decompression: {0}")]
    DeCompressError(#[from] DeCompressError),
    #[error("Failed Compression/Decompression: {0}")]
    DeEncryptError(#[from] DeEncryptError),
    #[error("Codec failed: {0}")]
    CodecError(#[from] CodecError),
}
