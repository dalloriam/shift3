use std::fs;
use std::io;
use std::path::Path;

use google_cloud::authorize::ApplicationCredentials;

use snafu::{ResultExt, Snafu};

/// Possible auth errors.
#[derive(Debug, Snafu)]
#[allow(missing_docs)] // Otherwise, cargo will ask to document each field of each error, which is a bit overkill.
pub enum AuthError {
    /// Error returned when the service account key file couldn't be read
    /// by the authentication provider.
    #[snafu(display("Failed to read service account key: {}", source))]
    FailedToReadServiceAccountKey { source: io::Error },

    #[snafu(display("Failed to parse service account JSON"))]
    FailedToParseJSON { source: serde_json::Error },
}

type Result<T> = std::result::Result<T, AuthError>;

/// Provides google cloud authentication with a simple API.
///
/// It requires less boilerplate than the bare `yup_oauth2` provider, and it implements
/// the `oauth::GetToken` trait, which means it can be used directly with Pubsub & Datastore clients.
// TODO: Update docs.
pub struct AuthProvider {
    credentials: ApplicationCredentials,
}

impl Clone for AuthProvider {
    fn clone(&self) -> Self {
        AuthProvider {
            credentials: self.credentials.clone(),
        }
    }
}

impl AuthProvider {
    /// Create an auth provider from a JSON file on disk.
    ///
    /// # Errors
    /// Returns an AuthError if the file cannot be read or if it doesn't contain
    /// valid JSON.
    pub fn from_json_file<P: AsRef<Path>>(path: P) -> Result<AuthProvider> {
        let file = fs::File::open(path.as_ref()).context(FailedToReadServiceAccountKey)?;

        let credentials: ApplicationCredentials =
            serde_json::from_reader(file).context(FailedToParseJSON)?;

        Ok(AuthProvider { credentials })
    }
}

impl From<AuthProvider> for ApplicationCredentials {
    fn from(provider: AuthProvider) -> ApplicationCredentials {
        provider.credentials
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::path::PathBuf;

    use tempfile::NamedTempFile;

    use super::AuthProvider;

    #[test]
    fn provider_from_json_file_exists() {
        let mut temp_json_file = NamedTempFile::new().unwrap();
        let auth_json = include_bytes!("test_data/valid_key.json");
        temp_json_file.write_all(auth_json.as_ref()).unwrap();

        // For now we only validate that the auth provider is created without errors.
        let _provider = AuthProvider::from_json_file(&temp_json_file.path()).unwrap();
    }

    #[test]
    fn provider_from_json_nonexistent_file() {
        let mock_path = PathBuf::from("/some/bad/path");
        assert!(AuthProvider::from_json_file(&mock_path).is_err());
    }

    #[test]
    fn provider_from_json_bad_json() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"bing bong").unwrap();

        assert!(AuthProvider::from_json_file(&temp_file.path()).is_err());
    }
}
