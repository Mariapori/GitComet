mod noop_backend;

pub use noop_backend::NoopBackend;

use gitcomet_core::services::GitBackend;
use gitcomet_core::services::{GitRepository, Result};
use std::path::Path;
use std::sync::Arc;

pub fn default_backend() -> Arc<dyn GitBackend> {
    Arc::new(NoopBackend)
}

pub fn open_repo(workdir: &Path) -> Result<Arc<dyn GitRepository>> {
    default_backend().open(workdir)
}

#[cfg(test)]
mod tests {
    use super::{default_backend, open_repo};
    use gitcomet_core::error::ErrorKind;
    use std::path::Path;

    #[test]
    fn default_backend_is_noop_and_reports_unsupported() {
        let backend = default_backend();
        let err = match backend.open(Path::new(".")) {
            Ok(_) => panic!("default backend should be noop without backend features"),
            Err(err) => err,
        };
        assert!(matches!(err.kind(), ErrorKind::Unsupported(_)));
    }

    #[test]
    fn open_repo_uses_default_backend() {
        let err = match open_repo(Path::new(".")) {
            Ok(_) => panic!("open_repo should fail via noop backend"),
            Err(err) => err,
        };
        assert!(matches!(err.kind(), ErrorKind::Unsupported(_)));
    }
}
