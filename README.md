# StegCloak

[![crates.io](https://img.shields.io/crates/v/stegcloak.svg)](https://crates.io/crates/stegcloak)

Hides secrets inside text by compressing and encrypting the secret before cloaking it
with special unicode invisible characters. It can be used to safely watermark strings,
invisible scripts on webpages, texts on social media, or for any other covert communication.
Completely invisible!

Inspired by the original javascript [stegcloak](https://github.com/KuroLabs/stegcloak).

This is incompatible with the original js stegcloak. But it can compile to wasm with the
`wasm` feature.

# Features

- Allows you to invisibly hide your secret inside regular text
- Protect your secret with password and HMAC integrity
- Encrypts your secret with AES-256-CTR
- Uses 6 invisible unicode characters that work in many places such as Gmail, WhatsApp, Telegram, Facebook, and more!
- Uses compression to minimize size of payload.
- Wasm compatible
- Fast🦀! Can hide the entire wikipedia source in ~201468 characters taking ~3.5ms in plaintext mode and ~7ms in encrypt mode

# Cargo Features

`wasm` - If you need wasm support, this feature's for you!

# Warning

This is currently under dev. Algorithm may be changed at any time, and previously encoded
messages may no longer be compatible with the new version.

Every effort has been made to be cryptographically secure, however, this _should not_ be
relied on for any sensitive or secure communications! Author absolves self from all possible
issues that could arise from usage of this software.

StegCloak doesn't solve the Alice-Bob-Warden problem, it's powerful only when people are not
looking for it and it helps you achieve that really well, given its invisible properties around
the web! It could be safely used for watermarking in forums, invisible tweets, social media etc.
Please don't use it when you know there's someone who is actively sniffing your data - looking at
the unicode characters through a data analysis tool. In that case, even though the secret encoded
cannot be deciphered, the fact lies that the Warden (middle-man) knows some secret communication
took place, because he would have noticed an unusual amount of special invisible characters.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT
LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY,
WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE
OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
