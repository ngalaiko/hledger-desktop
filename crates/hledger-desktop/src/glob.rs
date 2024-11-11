use std::path::{Path, PathBuf};

use iced::futures::StreamExt;

pub fn walk<'glob, P: wax::Pattern<'glob>>(path: &Path, glob: &'glob P) -> Walker<'glob, P> {
    Walker::new(path, glob)
}

pub struct Walker<'glob, P: wax::Pattern<'glob>> {
    start_path: PathBuf,
    fs: async_walkdir::WalkDir,
    pattern: &'glob P,
}

impl<'glob, P: wax::Pattern<'glob>> Walker<'glob, P> {
    fn new(path: &Path, glob: &'glob P) -> Self {
        Self {
            start_path: path.to_owned(),
            fs: async_walkdir::WalkDir::new(path),
            pattern: glob,
        }
    }

    fn matches(&self, path: &Path) -> bool {
        self.pattern
            .is_match(path.strip_prefix(&self.start_path).unwrap())
    }

    pub async fn next(&mut self) -> Result<Option<PathBuf>, async_walkdir::Error> {
        loop {
            match self.fs.next().await {
                Some(Ok(entry)) if self.matches(&entry.path()) => return Ok(Some(entry.path())),
                Some(Err(error)) => return Err(error),
                None => return Ok(None),
                _ => continue,
            }
        }
    }

    pub fn as_stream<'a>(
        &'a mut self,
    ) -> impl iced::futures::Stream<Item = Result<PathBuf, async_walkdir::Error>> + 'a
    where
        'a: 'glob,
    {
        iced::futures::stream::unfold(self, |self_| async {
            self_.next().await.transpose().map(|v| (v, self_))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use macro_rules_attribute::apply;

    async fn glob_walker_test(glob: &str, input: Vec<&str>, expected: Vec<&str>) {
        let temp_dir = tempfile::tempdir().unwrap();
        for p in input {
            let path = temp_dir.path().join(p);
            smol::fs::create_dir_all(path.parent().unwrap())
                .await
                .unwrap();
            smol::fs::write(path, b"").await.unwrap();
        }

        let glob = wax::any([glob]).unwrap();

        let mut paths = vec![];
        let mut entries = walk(temp_dir.path(), &glob);
        while let Some(e) = entries.next().await.unwrap() {
            paths.push(e);
        }

        let expected = expected
            .into_iter()
            .map(|e| temp_dir.path().join(e))
            .collect::<Vec<_>>();
        assert_eq!(paths, expected);
    }

    #[apply(smol_macros::test!)]
    async fn glob_walker_basic() {
        glob_walker_test("*.txt", vec!["a.txt", "b.bin"], vec!["a.txt"]).await;
        glob_walker_test("foo/*", vec!["foo/a", "bar/b"], vec!["foo/a"]).await;
        glob_walker_test(
            "foo/**/*",
            vec!["foo/bar/baz", "bar/b"],
            vec!["foo/bar", "foo/bar/baz"],
        )
        .await;
    }
}
