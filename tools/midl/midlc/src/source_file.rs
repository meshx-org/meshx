use std::path::Path;

/// A MIDL schema document.
#[derive(Debug, Clone)]
pub(crate) struct SourceFile<'src> {
    filename: &'src str,
    contents: String,
}

impl<'src> SourceFile<'src> {
    pub fn new(path: &'src Path) -> Result<Self, std::io::Error> {
        let contents = std::fs::read_to_string(path);

        match contents {
            Ok(contents) => Ok(Self {
                filename: path.file_name().unwrap().to_str().unwrap(),
                contents,
            }),
            Err(e) => Err(e),
        }
    }

    pub fn as_str(&self) -> &str {
        self.contents.as_str()
    }

    pub fn filename(&self) -> &str {
        self.filename
    }
}

#[derive(Debug)]
pub(crate) struct SourceManager<'files> {
    pub files: Vec<SourceFile<'files>>,
}

impl<'files> From<Vec<SourceFile<'files>>> for SourceManager<'files> {
    fn from(files: Vec<SourceFile<'files>>) -> Self {
        Self { files }
    }
}

impl<'files> SourceManager<'files> {
    pub(crate) fn get(&self, filename: &str) -> Option<&SourceFile<'_>> {
        self.files.iter().find(|f| f.filename == filename)
    }

    /// Enumerate over all source files in this collection.
    pub(crate) fn iter(&self) -> impl Iterator<Item = (SourceId, &SourceFile<'_>)> {
        self.files
            .iter()
            .enumerate()
            .map(|(file_idx, file)| (SourceId(file_idx), file))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceId(pub usize);

impl<'files> std::ops::Index<SourceId> for SourceManager<'files> {
    type Output = SourceFile<'files>;

    fn index(&self, index: SourceId) -> &Self::Output {
        &self.files[index.0]
    }
}
