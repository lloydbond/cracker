use std::fmt::Debug;
use std::path::Path;
use std::sync::Arc;
use tokio::{fs, io};

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    IoError(io::ErrorKind),
    CliExit,
}

pub async fn async_read_lines<P>(filename: P) -> Result<Arc<String>, Error>
where
    P: AsRef<Path> + Debug + Clone,
{
    let contents = fs::read_to_string(filename.clone())
        .await
        .map(Arc::new)
        .map_err(|error| Error::IoError(error.kind()))?;
    debug!("makefile ({:?}) loaded", filename.clone());

    Ok(contents)
}

#[cfg(test)]
mod tests {
    use std::io::ErrorKind;

    use super::*;

    #[tokio::test]
    async fn test_file_not_found() {
        let result: Result<Arc<String>, Error> = async_read_lines("non-existent.file").await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Error::IoError(ErrorKind::NotFound));
    }

    #[tokio::test]
    async fn test_empty_file() {
        let result: Result<Arc<String>, Error> =
            async_read_lines("tests/test_files/empty.txt").await;
        assert!(result.is_ok());

        let content: Arc<String> = result.unwrap();
        assert!(content.is_empty());
    }

    #[tokio::test]
    async fn test_single_line_file() {
        let result: Result<Arc<String>, Error> =
            async_read_lines("tests/test_files/single_line.txt").await;
        assert!(result.is_ok());
        let actual: Arc<String> = result.unwrap();
        let expected: Arc<String> = Arc::new("Hello World".to_string());
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_multiple_lines_file() {
        let result: Result<Arc<String>, Error> =
            async_read_lines("tests/test_files/multiple_lines.txt").await;
        assert!(result.is_ok());
        let actual: Arc<String> = result.unwrap();
        let expected: Arc<String> = Arc::new("Line 1\nLine 2\nLine 3".to_string());
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_file_with_empty_lines() {
        let result: Result<Arc<String>, Error> =
            async_read_lines("tests/test_files/empty_lines.txt").await;
        assert!(result.is_ok());
        let actual: Arc<String> = result.unwrap();
        let expected: Arc<String> = Arc::new("First\n\n\nLast".to_string());
        assert_eq!(actual, expected);
    }
}
