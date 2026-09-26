use super::IconName;

/// The icon a file's name suggests, by its extension.
pub(crate) fn file_icon(name: &str) -> IconName {
    let extension = name.rsplit_once('.').map(|(_, ext)| ext.to_lowercase());
    match extension.as_deref() {
        Some(
            "rs" | "ts" | "tsx" | "js" | "jsx" | "py" | "go" | "swift" | "c" | "h" | "cpp" | "java"
            | "kt" | "rb" | "php" | "css" | "html",
        ) => IconName::FileCode,
        Some("sh" | "zsh" | "bash" | "command") => IconName::FileTerminal,
        Some("toml" | "yaml" | "yml" | "ini" | "lock" | "env" | "conf") => IconName::FileCog,
        Some("json") => IconName::FileJson,
        Some("png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "heic" | "bmp" | "tiff") => {
            IconName::FileImage
        }
        Some("mp3" | "wav" | "flac" | "aac" | "m4a" | "ogg" | "aiff") => IconName::FileAudio,
        Some("mp4" | "mov" | "mkv" | "webm" | "avi") => IconName::FileVideoCamera,
        Some("zip" | "tar" | "gz" | "tgz" | "rar" | "7z") => IconName::FileArchive,
        Some("csv" | "xls" | "xlsx" | "numbers") => IconName::FileSpreadsheet,
        Some("ttf" | "otf" | "woff" | "woff2") => IconName::FileType,
        Some("md" | "txt" | "rtf" | "pdf" | "doc" | "docx" | "pages") => IconName::FileText,
        _ => IconName::File,
    }
}

#[cfg(test)]
mod tests {
    use super::{IconName, file_icon};

    #[test]
    fn a_name_picks_its_icon_by_its_last_extension() {
        assert_eq!(file_icon("main.RS"), IconName::FileCode);
        assert_eq!(file_icon("lift.PNG"), IconName::FileImage);
        assert_eq!(file_icon("notes.md"), IconName::FileText);
        assert_eq!(file_icon("backup.tar.gz"), IconName::FileArchive);
        assert_eq!(file_icon(".env"), IconName::FileCog);
        assert_eq!(file_icon("take.mov"), IconName::FileVideoCamera);
        assert_eq!(file_icon("Makefile"), IconName::File);
    }
}
