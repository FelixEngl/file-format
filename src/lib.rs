#![doc = include_str ! (concat ! (env ! ("CARGO_MANIFEST_DIR"), "/README.md"))]

use std::{
    fs::File,
    io::{Read, Result},
    path::Path,
};

/// A file format.
#[derive(Clone, Debug, PartialEq)]
pub struct FileFormat {
    media_type: String,
    extension: String,
}

impl FileFormat {
    /// Maximum number of bytes to read to detect the `FileFormat`.
    const MAX_BYTES: u64 = 36870;

    /// Creates a new `FileFormat` from a media type and an extension.
    fn new(media_type: &str, extension: &str) -> FileFormat {
        FileFormat {
            media_type: String::from(media_type),
            extension: String::from(extension),
        }
    }

    /// Determines `FileFormat` from bytes.
    ///
    /// If the `FileFormat` is not recognized, it will return the [default value].
    ///
    /// # Examples
    ///
    /// Detects from the first bytes of a PNG file:
    ///
    /// ```rust
    /// use file_format::FileFormat;
    ///
    /// let format = FileFormat::from_bytes(b"\x89\x50\x4E\x47\x0D\x0A\x1A\x0A");
    /// assert_eq!(format.media_type(), "image/png");
    /// assert_eq!(format.extension(), "png");
    ///```
    ///
    /// Detects from a zeroed buffer:
    ///
    /// ```rust
    /// use file_format::FileFormat;
    ///
    /// let format = FileFormat::from_bytes(&[0; 1000]);
    /// assert_eq!(format, FileFormat::default());
    /// assert_eq!(format.media_type(), "application/octet-stream");
    /// assert_eq!(format.extension(), "bin");
    ///```
    ///
    /// [default value]: FileFormat::default
    #[inline]
    pub fn from_bytes(bytes: &[u8]) -> FileFormat {
        FileFormat::from_signature(bytes).unwrap_or_default()
    }

    /// Determines `FileFormat` from a file.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use file_format::FileFormat;
    ///
    /// let format = FileFormat::from_file("fixtures/video/sample.mkv")?;
    /// assert_eq!(format.media_type(), "video/x-matroska");
    /// assert_eq!(format.extension(), "mkv");
    /// # Ok::<(), std::io::Error>(())
    ///```
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<FileFormat> {
        let file = File::open(path)?;
        let mut handle = file.take(FileFormat::MAX_BYTES);
        let mut buffer = [0; FileFormat::MAX_BYTES as usize];
        let read = handle.read(&mut buffer)?;
        Ok(FileFormat::from_bytes(&buffer[0..read]))
    }

    /// Returns the media type (formerly known as MIME type) of the `FileFormat`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use file_format::FileFormat;
    ///
    /// let format = FileFormat::from_file("fixtures/application/sample.zst")?;
    /// assert_eq!(format.media_type(), "application/zstd");
    /// # Ok::<(), std::io::Error>(())
    ///```
    #[inline]
    pub fn media_type(&self) -> &str {
        &self.media_type
    }

    /// Returns the preferred extension of the `FileFormat`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use file_format::FileFormat;
    ///
    /// let format = FileFormat::from_file("fixtures/video/sample.wmv")?;
    /// assert_eq!(format.extension(), "wmv");
    /// # Ok::<(), std::io::Error>(())
    ///```
    #[inline]
    pub fn extension(&self) -> &str {
        &self.extension
    }
}

impl Default for FileFormat {
    /// Returns the default `FileFormat` which corresponds to arbitrary binary data.
    #[inline]
    fn default() -> FileFormat {
        FileFormat::new("application/octet-stream", "bin")
    }
}

/// Generates [`FileFormat::from_signature`] using a database described in YAML-like format.
macro_rules! signatures {
    {
        $(
            -   media_type: $media_type:literal
                extension: $extension:literal
                signatures:
                $(
                    -   parts:
                    $(
                        -   offset: $offset:literal
                            value: $signature:literal
                    )+
                )+
        )*
    } => {
        impl FileFormat {
            /// Determines `FileFormat` from bytes by checking the signature.
            fn from_signature(bytes: &[u8]) -> Option<FileFormat> {
                $(
                    if
                        $(
                            $(
                                bytes.len() >= $offset + $signature.len()
                                    && &bytes[$offset..$offset + $signature.len()] == $signature
                            )&&*
                        )||*
                    { return Some(FileFormat::new($media_type, $extension)); }
                )*
                None
            }
        }
    };
}

signatures! {
  // 59-byte signatures
  - media_type: "application/vnd.oasis.opendocument.presentation"
    extension: "odp"
    signatures:
      - parts:
        - offset: 0
          value: b"\x50\x4B\x03\x04"
        - offset: 30
          value: b"mimetype"
        - offset: 38
          value: b"application/vnd.oasis.opendocument.presentation"

  // 58-byte signatures
  - media_type: "application/vnd.oasis.opendocument.spreadsheet"
    extension: "ods"
    signatures:
      - parts:
        - offset: 0
          value: b"\x50\x4B\x03\x04"
        - offset: 30
          value: b"mimetype"
        - offset: 38
          value: b"application/vnd.oasis.opendocument.spreadsheet"

  // 55-byte signatures
  - media_type: "application/vnd.oasis.opendocument.graphics"
    extension: "odg"
    signatures:
      - parts:
        - offset: 0
          value: b"\x50\x4B\x03\x04"
        - offset: 30
          value: b"mimetype"
        - offset: 38
          value: b"application/vnd.oasis.opendocument.graphics"

  // 51-byte signatures
  - media_type: "application/vnd.oasis.opendocument.text"
    extension: "odt"
    signatures:
      - parts:
        - offset: 0
          value: b"\x50\x4B\x03\x04"
        - offset: 30
          value: b"mimetype"
        - offset: 38
          value: b"application/vnd.oasis.opendocument.text"

  // 39-byte signatures
  - media_type: "application/x-virtualbox-vdi"
    extension: "vdi"
    signatures:
      - parts:
        - offset: 0
          value: b"<<< Oracle VM VirtualBox Disk Image >>>"

  // 32-byte signatures
  - media_type: "application/epub+zip"
    extension: "epub"
    signatures:
      - parts:
        - offset: 0
          value: b"\x50\x4B\x03\x04"
        - offset: 30
          value: b"mimetype"
        - offset: 38
          value: b"application/epub+zip"

  - media_type: "application/vnd.sketchup.skp"
    extension: "skp"
    signatures:
      - parts:
        - offset: 0
          value: b"\xFF\xFE\xFF\x0E\x53\x00\x6B\x00\x65\x00\x74\x00\x63\x00\x68\x00"
        - offset: 16
          value: b"\x55\x00\x70\x00\x20\x00\x4D\x00\x6F\x00\x64\x00\x65\x00\x6C\x00"

  // 21-byte signatures
  - media_type: "application/vnd.debian.binary-package"
    extension: "deb"
    signatures:
      - parts:
        - offset: 0
          value: b"\x21\x3C\x61\x72\x63\x68\x3E\x0A"
        - offset: 8
          value: b"debian-binary"

  // 16-byte signatures
  - media_type: "application/vnd.sqlite3"
    extension: "sqlite"
    signatures:
      - parts:
        - offset: 0
          value: b"\x53\x51\x4C\x69\x74\x65\x20\x66\x6F\x72\x6D\x61\x74\x20\x33\x00"

  - media_type: "application/x-indesign"
    extension: "indd"
    signatures:
      - parts:
        - offset: 0
          value: b"\x06\x06\xED\xF5\xD8\x1D\x46\xE5\xBD\x31\xEF\xE7\xFE\x74\xB7\x1D"

  // 14-byte signatures
  - media_type: "application/mxf"
    extension: "mxf"
    signatures:
      - parts:
        - offset: 0
          value: b"\x06\x0E\x2B\x34\x02\x05\x01\x01\x0D\x01\x02\x01\x01\x02"

  // 12-byte signatures
  - media_type: "audio/opus"
    extension: "opus"
    signatures:
      - parts:
        - offset: 0
          value: b"OggS"
        - offset: 28
          value: b"OpusHead"

  - media_type: "image/apng"
    extension: "apng"
    signatures:
      - parts:
        - offset: 0
          value: b"\x89\x50\x4E\x47\x0D\x0A\x1A\x0A"
        - offset: 0x25
          value: b"acTL"

  - media_type: "image/jpeg"
    extension: "jpg"
    signatures:
      - parts:
        - offset: 0
          value: b"\xFF\xD8\xFF\xE0\x00\x10\x4A\x46\x49\x46\x00\x01"
      - parts:
        - offset: 0
          value: b"\xFF\xD8\xFF\xE1"
        - offset: 6
          value: b"\x45\x78\x69\x66\x00\x00"
      - parts:
        - offset: 0
          value: b"\xFF\xD8\xFF\xDB"
      - parts:
        - offset: 0
          value: b"\xFF\xD8\xFF\xEE"

  - media_type: "image/jxl"
    extension: "jxl"
    signatures:
      - parts:
        - offset: 0
          value: b"\x00\x00\x00\x0C\x4A\x58\x4C\x20\x0D\x0A\x87\x0A"
      - parts:
        - offset: 0
          value: b"\xFF\x0A"

  - media_type: "image/ktx"
    extension: "ktx"
    signatures:
      - parts:
        - offset: 0
          value: b"\xAB\x4B\x54\x58\x20\x31\x31\xBB\x0D\x0A\x1A\x0A"

  - media_type: "image/ktx2"
    extension: "ktx2"
    signatures:
      - parts:
        - offset: 0
          value: b"\xAB\x4B\x54\x58\x20\x32\x30\xBB\x0D\x0A\x1A\x0A"

  - media_type: "video/x-matroska"
    extension: "mkv"
    signatures:
      - parts:
        - offset: 0
          value: b"\x1A\x45\xDF\xA3"
        - offset: 24
          value: b"matroska"

  // 10-byte signatures
  - media_type: "audio/ogg"
    extension: "ogg"
    signatures:
      - parts:
        - offset: 0
          value: b"OggS"
        - offset: 29
          value: b"vorbis"

  - media_type: "image/fits"
    extension: "fits"
    signatures:
      - parts:
        - offset: 0
          value: b"\x53\x49\x4D\x50\x4C\x45\x20\x20\x3D\x20"

  - media_type: "video/ogg"
    extension: "ogv"
    signatures:
      - parts:
        - offset: 0
          value: b"OggS"
        - offset: 29
          value: b"theora"

  - media_type: "video/quicktime"
    extension: "mov"
    signatures:
      - parts:
        - offset: 0
          value: b"\x00\x00\x00\x14"
        - offset: 4
          value: b"ftypqt"

  - media_type: "video/x-ms-asf"
    extension: "wmv"
    signatures:
      - parts:
        - offset: 0
          value: b"\x30\x26\xB2\x75\x8E\x66\xCF\x11\xA6\xD9"

  // 9-byte signatures
  - media_type: "application/x-gameboy-color-rom"
    extension: "gbc"
    signatures:
      - parts:
        - offset: 0x104
          value: b"\xCE\xED\x66\x66\xCC\x0D\x00\x0B"
        - offset: 0x143
          value: b"\x80"
      - parts:
        - offset: 0x104
          value: b"\xCE\xED\x66\x66\xCC\x0D\x00\x0B"
        - offset: 0x143
          value: b"\xC0"

  - media_type: "application/x-lzop"
    extension: "lzo"
    signatures:
      - parts:
        - offset: 0
          value: b"\x89\x4C\x5A\x4F\x00\x0D\x0A\x1A\x0A"

  - media_type: "audio/ogg"
    extension: "spx"
    signatures:
      - parts:
        - offset: 0
          value: b"OggS"
        - offset: 28
          value: b"Speex"

  - media_type: "image/x-olympus-orf"
    extension: "orf"
    signatures:
      - parts:
        - offset: 0
          value: b"\x49\x49\x52\x4F\x08\x00\x00\x00\x18"

  - media_type: "video/ogg"
    extension: "ogm"
    signatures:
      - parts:
        - offset: 0
          value: b"OggS"
        - offset: 29
          value: b"video"

  // 8-byte signatures
  - media_type: "application/vnd.rar"
    extension: "rar"
    signatures:
      - parts:
        - offset: 0
          value: b"\x52\x61\x72\x21\x1A\x07\x01\x00"
      - parts:
        - offset: 0
          value: b"\x52\x61\x72\x21\x1A\x07\x00"

  - media_type: "application/x-gameboy-rom"
    extension: "gb"
    signatures:
      - parts:
        - offset: 0x104
          value: b"\xCE\xED\x66\x66\xCC\x0D\x00\x0B"

  - media_type: "application/x-gba-rom"
    extension: "gba"
    signatures:
      - parts:
        - offset: 4
          value: b"\x24\xFF\xAE\x51\x69\x9A\xA2\x21"

  - media_type: "application/x-mobipocket-ebook"
    extension: "mobi"
    signatures:
      - parts:
        - offset: 60
          value: b"BOOKMOBI"

  - media_type: "application/x-ms-shortcut"
    extension: "lnk"
    signatures:
      - parts:
        - offset: 0
          value: b"\x4C\x00\x00\x00\x01\x14\x02\x00"

  - media_type: "application/x-n64-rom"
    extension: "z64"
    signatures:
      - parts:
        - offset: 0
          value: b"\x80\x37\x12\x40\x00\x00\x00\x0F"
      - parts:
        - offset: 0
          value: b"\x37\x80\x40\x12\x00\x00\x0F\x00"
      - parts:
        - offset: 0
          value: b"\x12\x40\x80\x37\x00\x0F\x00\x00"
      - parts:
        - offset: 0
          value: b"\x40\x12\x37\x80\x0F\x00\x00\x00"

  - media_type: "application/x-nintendo-ds-rom"
    extension: "nds"
    signatures:
      - parts:
        - offset: 0xC0
          value: b"\x24\xFF\xAE\x51\x69\x9A\xA2\x21"
      - parts:
        - offset: 0xC0
          value: b"\xC8\x60\x4F\xE2\x01\x70\x8F\xE2"

  - media_type: "application/x-ole-storage"
    extension: "msi"
    signatures:
      - parts:
        - offset: 0
          value: b"\xD0\xCF\x11\xE0\xA1\xB1\x1A\xE1"

  - media_type: "application/x-tar"
    extension: "tar"
    signatures:
      - parts:
        - offset: 257
          value: b"\x75\x73\x74\x61\x72\x00\x30\x30"
      - parts:
        - offset: 257
          value: b"\x75\x73\x74\x61\x72\x20\x20\x00"

  - media_type: "audio/aiff"
    extension: "aif"
    signatures:
      - parts:
        - offset: 0
          value: b"FORM"
        - offset: 8
          value: b"AIFF"

  - media_type: "audio/ogg"
    extension: "oga"
    signatures:
      - parts:
        - offset: 0
          value: b"OggS"
        - offset: 29
          value: b"FLAC"

  - media_type: "audio/vnd.wave"
    extension: "wav"
    signatures:
      - parts:
        - offset: 0
          value: b"RIFF"
        - offset: 8
          value: b"WAVE"

  - media_type: "image/avif"
    extension: "avif"
    signatures:
      - parts:
        - offset: 4
          value: b"ftypavif"

  - media_type: "image/heic"
    extension: "heic"
    signatures:
      - parts:
        - offset: 4
          value: b"ftypheic"
      - parts:
        - offset: 4
          value: b"ftypheix"

  - media_type: "image/png"
    extension: "png"
    signatures:
      - parts:
        - offset: 0
          value: b"\x89\x50\x4E\x47\x0D\x0A\x1A\x0A"

  - media_type: "image/webp"
    extension: "webp"
    signatures:
      - parts:
        - offset: 0
          value: b"RIFF"
        - offset: 8
          value: b"WEBP"

  - media_type: "image/x-xcf"
    extension: "xcf"
    signatures:
      - parts:
        - offset: 0
          value: b"gimp xcf"

  - media_type: "video/avi"
    extension: "avi"
    signatures:
      - parts:
        - offset: 0
          value: b"RIFF"
        - offset: 8
          value: b"\x41\x56\x49\x20"

  - media_type: "video/mp4"
    extension: "mp4"
    signatures:
      - parts:
        - offset: 4
          value: b"ftypavc1"
      - parts:
        - offset: 4
          value: b"ftypdash"
      - parts:
        - offset: 4
          value: b"ftypiso2"
      - parts:
        - offset: 4
          value: b"ftypiso3"
      - parts:
        - offset: 4
          value: b"ftypiso4"
      - parts:
        - offset: 4
          value: b"ftypiso5"
      - parts:
        - offset: 4
          value: b"ftypiso6"
      - parts:
        - offset: 4
          value: b"ftypisom"
      - parts:
        - offset: 4
          value: b"ftypmmp4"
      - parts:
        - offset: 4
          value: b"ftypmp41"
      - parts:
        - offset: 4
          value: b"ftypmp42"
      - parts:
        - offset: 4
          value: b"ftypmp4v"
      - parts:
        - offset: 4
          value: b"ftypmp71"
      - parts:
        - offset: 4
          value: b"ftypMSNV"
      - parts:
        - offset: 4
          value: b"ftypNDAS"
      - parts:
        - offset: 4
          value: b"ftypNDSC"
      - parts:
        - offset: 4
          value: b"ftypNDSH"
      - parts:
        - offset: 4
          value: b"ftypNDSM"
      - parts:
        - offset: 4
          value: b"ftypNDSP"
      - parts:
        - offset: 4
          value: b"ftypNDSS"
      - parts:
        - offset: 4
          value: b"ftypNDXC"
      - parts:
        - offset: 4
          value: b"ftypNDXH"
      - parts:
        - offset: 4
          value: b"ftypNDXM"
      - parts:
        - offset: 4
          value: b"ftypNDXP"
      - parts:
        - offset: 4
          value: b"ftypF4V"
      - parts:
        - offset: 4
          value: b"ftypF4P"

  - media_type: "video/webm"
    extension: "webm"
    signatures:
      - parts:
        - offset: 0
          value: b"\x1A\x45\xDF\xA3"
        - offset: 24
          value: b"webm"

  // 7-byte signatures
  - media_type: "application/x-archive"
    extension: "ar"
    signatures:
      - parts:
        - offset: 0
          value: b"!<arch>"

  - media_type: "application/x-blender"
    extension: "blend"
    signatures:
      - parts:
        - offset: 0
          value: b"BLENDER"

  - media_type: "audio/x-m4a"
    extension: "m4a"
    signatures:
      - parts:
        - offset: 4
          value: b"ftypM4A"

  - media_type: "image/jp2"
    extension: "jp2"
    signatures:
      - parts:
        - offset: 16
          value: b"ftypjp2"

  - media_type: "video/3gpp"
    extension: "3gp"
    signatures:
      - parts:
        - offset: 4
          value: b"ftyp3gp"

  - media_type: "video/3gpp2"
    extension: "3g2"
    signatures:
      - parts:
        - offset: 4
          value: b"ftyp3g2"

  - media_type: "video/x-m4v"
    extension: "m4v"
    signatures:
      - parts:
        - offset: 4
          value: b"ftypM4V"

  // 6-byte signatures
  - media_type: "application/x-7z-compressed"
    extension: "7z"
    signatures:
      - parts:
        - offset: 0
          value: b"\x37\x7A\xBC\xAF\x27\x1C"

  - media_type: "application/x-xz"
    extension: "xz"
    signatures:
      - parts:
        - offset: 0
          value: b"\xFD\x37\x7A\x58\x5A\x00"

  - media_type: "image/gif"
    extension: "gif"
    signatures:
      - parts:
        - offset: 0
          value: b"GIF87a"
      - parts:
        - offset: 0
          value: b"GIF89a"

  // 5-byte signatures
  - media_type: "application/pdf"
    extension: "pdf"
    signatures:
      - parts:
        - offset: 0
          value: b"%PDF-"

  - media_type: "application/vnd.ms-fontobject"
    extension: "eot"
    signatures:
      - parts:
        - offset: 8
          value: b"\x00\x00\x01"
        - offset: 34
          value: b"\x4C\x50"
      - parts:
        - offset: 8
          value: b"\x01\x00\x02"
        - offset: 34
          value: b"\x4C\x50"
      - parts:
        - offset: 8
          value: b"\x02\x00\x02"
        - offset: 34
          value: b"\x4C\x50"

  - media_type: "application/x-iso9660-image"
    extension: "iso"
    signatures:
      - parts:
        - offset: 0x8001
          value: b"CD001"
      - parts:
        - offset: 0x8801
          value: b"CD001"
      - parts:
        - offset: 0x9001
          value: b"CD001"

  - media_type: "audio/amr"
    extension: "amr"
    signatures:
      - parts:
        - offset: 0
          value: b"#!AMR"

  - media_type: "font/otf"
    extension: "otf"
    signatures:
      - parts:
        - offset: 0
          value: b"\x4F\x54\x54\x4F\x00"

  - media_type: "font/ttf"
    extension: "ttf"
    signatures:
      - parts:
        - offset: 0
          value: b"\x00\x01\x00\x00\x00"

  // 4-byte signatures
  - media_type: "application/dicom"
    extension: "dcm"
    signatures:
      - parts:
        - offset: 128
          value: b"\x44\x49\x43\x4D"

  - media_type: "application/java-vm"
    extension: "class"
    signatures:
      - parts:
        - offset: 0
          value: b"\xCA\xFE\xBA\xBE"

  - media_type: "application/ogg"
    extension: "ogx"
    signatures:
      - parts:
        - offset: 0
          value: b"OggS"

  - media_type: "application/vnd.android.dex"
    extension: "dex"
    signatures:
      - parts:
        - offset: 0
          value: b"\x64\x65\x78\x0a"

  - media_type: "application/vnd.ms-cab-compressed"
    extension: "cab"
    signatures:
      - parts:
        - offset: 0
          value: b"MSCF"
      - parts:
        - offset: 0
          value: b"ISc("

  - media_type: "application/vnd.tcpdump.pcap"
    extension: "pcap"
    signatures:
      - parts:
        - offset: 0
          value: b"\xA1\xB2\xC3\xD4"
      - parts:
        - offset: 0
          value: b"\xD4\xC3\xB2\xA1"

  - media_type: "application/wasm"
    extension: "wasm"
    signatures:
      - parts:
        - offset: 0
          value: b"\x00\x61\x73\x6D"

  - media_type: "application/x-esri-shape"
    extension: "shp"
    signatures:
      - parts:
        - offset: 0
          value: b"\x00\x00\x27\x0A"

  - media_type: "application/x-executable"
    extension: "elf"
    signatures:
      - parts:
        - offset: 0
          value: b"\x7F\x45\x4C\x46"

  - media_type: "application/x-google-chrome-extension"
    extension: "crx"
    signatures:
      - parts:
        - offset: 0
          value: b"Cr24"

  - media_type: "application/x-lrzip"
    extension: "lrz"
    signatures:
      - parts:
        - offset: 0
          value: b"LRZI"

  - media_type: "application/x-lz4"
    extension: "lz4"
    signatures:
      - parts:
        - offset: 0
          value: b"\x04\x22\x4D\x18"

  - media_type: "application/x-lzip"
    extension: "lz"
    signatures:
      - parts:
        - offset: 0
          value: b"LZIP"

  - media_type: "application/x-nintendo-nes-rom"
    extension: "nes"
    signatures:
      - parts:
        - offset: 0
          value: b"\x4E\x45\x53\x1A"

  - media_type: "application/x-pcapng"
    extension: "pcapng"
    signatures:
      - parts:
        - offset: 0
          value: b"\x0A\x0D\x0D\x0A"

  - media_type: "application/x-rpm"
    extension: "rpm"
    signatures:
      - parts:
        - offset: 0
          value: b"\xED\xAB\xEE\xDB"

  - media_type: "application/x-xar"
    extension: "xar"
    signatures:
      - parts:
        - offset: 0
          value: b"xar!"

  - media_type: "application/zip"
    extension: "zip"
    signatures:
      - parts:
        - offset: 0
          value: b"\x50\x4B\x03\x04"
      - parts:
        - offset: 0
          value: b"\x50\x4B\x05\x06"
      - parts:
        - offset: 0
          value: b"\x50\x4B\x07\x08"

  - media_type: "application/zstd"
    extension: "zst"
    signatures:
      - parts:
        - offset: 0
          value: b"\x28\xB5\x2F\xFD"

  - media_type: "audio/basic"
    extension: "au"
    signatures:
      - parts:
        - offset: 0
          value: b".snd"

  - media_type: "audio/midi"
    extension: "mid"
    signatures:
      - parts:
        - offset: 0
          value: b"MThd"

  - media_type: "audio/wavpack"
    extension: "wv"
    signatures:
      - parts:
        - offset: 0
          value: b"wvpk"

  - media_type: "audio/x-ape"
    extension: "ape"
    signatures:
      - parts:
        - offset: 0
          value: b"MAC "


  - media_type: "audio/x-flac"
    extension: "flac"
    signatures:
      - parts:
        - offset: 0
          value: b"fLaC"

  - media_type: "audio/x-musepack"
    extension: "mpc"
    signatures:
      - parts:
        - offset: 0
          value: b"MPCK"
      - parts:
        - offset: 0
          value: b"MP+"

  - media_type: "font/woff"
    extension: "woff"
    signatures:
      - parts:
        - offset: 0
          value: b"wOFF"

  - media_type: "font/woff2"
    extension: "woff2"
    signatures:
      - parts:
        - offset: 0
          value: b"wOF2"

  - media_type: "image/bpg"
    extension: "bpg"
    signatures:
      - parts:
        - offset: 0
          value: b"\x42\x50\x47\xFB"

  - media_type: "image/cineon"
    extension: "cin"
    signatures:
      - parts:
        - offset: 0
          value: b"\x80\x2A\x5F\xD7"

  - media_type: "image/flif"
    extension: "flif"
    signatures:
      - parts:
        - offset: 0
          value: b"FLIF"

  - media_type: "image/icns"
    extension: "icns"
    signatures:
      - parts:
        - offset: 0
          value: b"icns"

  - media_type: "image/tiff"
    extension: "tiff"
    signatures:
      - parts:
        - offset: 0
          value: b"MM\x00*"
      - parts:
        - offset: 0
          value: b"II*\x00"

  - media_type: "image/vnd.adobe.photoshop"
    extension: "psd"
    signatures:
      - parts:
        - offset: 0
          value: b"8BPS"

  - media_type: "image/wmf"
    extension: "wmf"
    signatures:
      - parts:
        - offset: 0
          value: b"\xD7\xCD\xC6\x9A"
      - parts:
        - offset: 0
          value: b"\x02\x00\x09\x00"
      - parts:
        - offset: 0
          value: b"\x01\x00\x09\x00"

  - media_type: "image/x-dpx"
    extension: "dpx"
    signatures:
      - parts:
        - offset: 0
          value: b"SDPX"
      - parts:
        - offset: 0
          value: b"XPDS"

  - media_type: "image/x-exr"
    extension: "exr"
    signatures:
      - parts:
        - offset: 0
          value: b"\x76\x2F\x31\x01"

  - media_type: "image/x-icon"
    extension: "ico"
    signatures:
      - parts:
        - offset: 0
          value: b"\x00\x00\x01\x00"

  - media_type: "model/gltf-binary"
    extension: "glb"
    signatures:
      - parts:
        - offset: 0
          value: b"glTF"

  - media_type: "video/mpeg"
    extension: "mpg"
    signatures:
      - parts:
        - offset: 0
          value: b"\x00\x00\x01\xBA"
      - parts:
        - offset: 0
          value: b"\x00\x00\x01\xB3"

  - media_type: "video/x-flv"
    extension: "flv"
    signatures:
      - parts:
        - offset: 0
          value: b"\x46\x4C\x56\x01"

  // 3-byte signatures
  - media_type: "application/x-bzip2"
    extension: "bz2"
    signatures:
      - parts:
        - offset: 0
          value: b"BZh"

  - media_type: "application/x-shockwave-flash"
    extension: "swf"
    signatures:
      - parts:
        - offset: 0
          value: b"\x43\x57\x53"
      - parts:
        - offset: 0
          value: b"\x46\x57\x53"

  - media_type: "audio/mpeg"
    extension: "mp3"
    signatures:
      - parts:
        - offset: 0
          value: b"ID3"

  - media_type: "image/jxr"
    extension: "jxr"
    signatures:
      - parts:
        - offset: 0
          value: b"\x49\x49\xBC"

  // 2-byte signatures
  - media_type: "application/gzip"
    extension: "gz"
    signatures:
      - parts:
        - offset: 0
          value: b"\x1F\x8B"

  - media_type: "application/x-apple-diskimage"
    extension: "dmg"
    signatures:
      - parts:
        - offset: 0
          value: b"\x78\x01"

  - media_type: "application/x-compress"
    extension: "Z"
    signatures:
      - parts:
        - offset: 0
          value: b"\x1F\xA0"
      - parts:
        - offset: 0
          value: b"\x1F\x9D"

  - media_type: "application/x-msdownload"
    extension: "exe"
    signatures:
      - parts:
        - offset: 0
          value: b"MZ"

  - media_type: "audio/aac"
    extension: "aac"
    signatures:
      - parts:
        - offset: 0
          value: b"\xFF\xF1"
      - parts:
        - offset: 0
          value: b"\xFF\xF9"

  - media_type: "audio/vnd.dolby.dd-raw"
    extension: "ac3"
    signatures:
      - parts:
        - offset: 0
          value: b"\x0B\x77"

  - media_type: "image/bmp"
    extension: "bmp"
    signatures:
      - parts:
        - offset: 0
          value: b"BM"

  - media_type: "video/mp2t"
    extension: "m2ts"
    signatures:
      - parts:
        - offset: 0
          value: b"\x47"
        - offset: 188
          value: b"\x47"
      - parts:
        - offset: 4
          value: b"\x47"
        - offset: 196
          value: b"\x47"
}
