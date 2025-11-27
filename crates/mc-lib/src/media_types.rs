use std::path::Path;

/// 媒体文件类型
#[derive(Debug, Clone, PartialEq)]
pub enum MediaType {
    Image,
    Video,
    Audio,
}

/// 媒体文件信息
#[derive(Debug, Clone)]
pub struct MediaInfo {
    pub media_type: MediaType,
    pub extension: String, // 大写形式，如 "JPG"
}

/// 根据文件路径获取媒体信息
pub fn get_media_info(path: &Path) -> Option<MediaInfo> {
    let extension = path.extension()?.to_str()?.to_lowercase();

    if is_image_extension(&extension) {
        Some(MediaInfo {
            media_type: MediaType::Image,
            extension: extension.to_uppercase(),
        })
    } else if is_video_extension(&extension) {
        Some(MediaInfo {
            media_type: MediaType::Video,
            extension: extension.to_uppercase(),
        })
    } else if is_audio_extension(&extension) {
        Some(MediaInfo {
            media_type: MediaType::Audio,
            extension: extension.to_uppercase(),
        })
    } else {
        None
    }
}

/// 检查是否为图片文件扩展名
pub fn is_image_extension(ext: &str) -> bool {
    matches!(
        ext,
        // 常规图片格式
        "jpg" | "jpeg" | "png" | "gif" | "tiff" | "tif" | "bmp" | "webp" | "heic" | "heif" |
        // RAW 格式
        "nef" | "nrw" |  // Nikon
        "cr2" | "cr3" | "crw" |  // Canon
        "arw" | "srf" | "sr2" |  // Sony
        "dng" |  // Adobe
        "orf" |  // Olympus
        "pef" |  // Pentax
        "raf" |  // Fujifilm
        "rw2" |  // Panasonic
        "3fr" |  // Hasselblad
        "iiq" |  // Phase One
        "mef" |  // Mamiya
        "mos" |  // Leaf
        "erf" |  // Epson
        "k25" | "kdc" | "dcr" | "dcs" // Kodak
    )
}

/// 检查是否为视频文件扩展名
pub fn is_video_extension(ext: &str) -> bool {
    matches!(
        ext,
        "mp4"
            | "m4v"
            | "mov"
            | "qt"
            | "avi"
            | "mkv"
            | "webm"
            | "wmv"
            | "flv"
            | "f4v"
            | "mts"
            | "m2ts"
            | "3gp"
            | "3g2"
            | "mpg"
            | "mpeg"
            | "mpe"
            | "mpv"
            | "ogv"
            | "vob"
    )
}

/// 检查是否为音频文件扩展名
pub fn is_audio_extension(ext: &str) -> bool {
    matches!(
        ext,
        // 无损格式
        "flac" | "wav" | "aiff" | "alac" | "ape" |
        // 有损格式
        "mp3" | "aac" | "m4a" | "ogg" | "oga" | "opus" | "wma" |
        // 其他
        "m4b" | "amr"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_extensions() {
        assert!(is_image_extension("jpg"));
        assert!(is_image_extension("nef"));
        assert!(is_image_extension("cr2"));
        assert!(!is_image_extension("mp4"));
    }

    #[test]
    fn test_video_extensions() {
        assert!(is_video_extension("mp4"));
        assert!(is_video_extension("mov"));
        assert!(!is_video_extension("jpg"));
    }

    #[test]
    fn test_audio_extensions() {
        assert!(is_audio_extension("mp3"));
        assert!(is_audio_extension("flac"));
        assert!(!is_audio_extension("jpg"));
    }
}
