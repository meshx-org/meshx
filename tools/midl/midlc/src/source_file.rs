/// A MIDL schema document.
#[derive(Debug, Clone)]
pub(crate) struct SourceFile<'src> {
    filename: &'src str,
    contents: String,
}

impl<'src> SourceFile<'src> {
    pub fn new(filename: &'src str, contents: String) -> Self {
        Self { filename, contents }
    }

    pub fn as_str(&self) -> &str {
        self.contents.as_str()
    }

    pub fn filename(&self) -> &str {
        self.filename
    }
}

pub(crate) struct SourceFiles<'files> {
    files: Vec<SourceFile<'files>>,
}

impl<'files> From<Vec<SourceFile<'files>>> for SourceFiles<'files> {
    fn from(files: Vec<SourceFile<'files>>) -> Self {
        Self { files }
    }
}

impl<'files> SourceFiles<'files> {
    pub(crate) fn add(&mut self, filename: &'files str, contents: String) {
        self.files.push(SourceFile::new(filename, contents));
    }

    pub(crate) fn get(&self, filename: &str) -> Option<&SourceFile<'_>> {
        self.files.iter().find(|f| f.filename == filename)
    }

    /// Enumerate over all source files in this collection.
    pub(crate) fn iter_sources(&self) -> impl Iterator<Item = (SourceId, &SourceFile<'_>)> {
        self.files
            .iter()
            .enumerate()
            .map(|(file_idx, file)| (SourceId(file_idx), file))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceId(pub usize);

impl<'files> std::ops::Index<SourceId> for SourceFiles<'files> {
    type Output = SourceFile<'files>;

    fn index(&self, index: SourceId) -> &Self::Output {
        &self.files[index.0]
    }
}
