use std::error::Error;
use std::io;
use std::path::Path;

use hyper::Client;

use snafu::{ResultExt, Snafu};

use yup_oauth2::{GetToken, ServiceAccountAccess, Token};

use crate::https;

/// Possible auth errors.
#[derive(Debug, Snafu)]
#[allow(missing_docs)] // Otherwise, cargo will ask to document each field of each error, which is a bit overkill.
pub enum AuthError {
    /// Error returned when the service account key file couldn't be read
    /// by the authentication provider.
    #[snafu(display("Failed to read service account key: {}", source))]
    FailedToReadServiceAccountKey { source: io::Error },
}

type Result<T> = std::result::Result<T, AuthError>;

/// Provides google cloud authentication with a simple API.
///
/// It requires less boilerplate than the bare `yup_oauth2` provider, and it implements
/// the `oauth::GetToken` trait, which means it can be used directly with Pubsub & Datastore clients.
pub struct AuthProvider {
    access_manager: ServiceAccountAccess<Client>,
}

impl AuthProvider {
    /// Create an auth provider from a JSON file on disk.
    ///
    /// # Errors
    /// Returns an AuthError if the file cannot be read or if it doesn't contain
    /// valid JSON.
    pub fn from_json_file<P: AsRef<Path>>(path: P) -> Result<AuthProvider> {
        // This is needed because `yup_oauth2` doesn't support `std::Path`...
        let path_str = String::from(path.as_ref().to_string_lossy());

        let secret = yup_oauth2::service_account_key_from_file(&path_str)
            .context(FailedToReadServiceAccountKey)?;

        let access_manager = ServiceAccountAccess::new(secret, https::new_tls_client());

        Ok(AuthProvider { access_manager })
    }
}

impl GetToken for AuthProvider {
    fn token<'b, I, T>(&mut self, scopes: I) -> std::result::Result<Token, Box<dyn Error>>
    where
        T: AsRef<str> + Ord + 'b,
        I: IntoIterator<Item = &'b T>,
    {
        self.access_manager.token(scopes)
    }

    fn api_key(&mut self) -> Option<String> {
        self.access_manager.api_key()
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
