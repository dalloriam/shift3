use protocol::ActionManifest;

use crate::Error;

pub trait ActionPlugin {
    fn execute_action(&self, manifest: ActionManifest) -> Result<(), Error>;
    fn get_type(&self) -> &str;
}
