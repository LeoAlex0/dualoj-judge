use uuid::Uuid;

use super::ControlService;

impl ControlService {
    pub(crate) fn get_image_url(&self, uuid: &Uuid) -> String {
        format!(
            "{}/{}/{}",
            self.registry.registry_url, self.registry.username, uuid
        )
    }
}