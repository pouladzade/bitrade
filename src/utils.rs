/// Generates a unique trade ID
pub fn generate_uuid_id() -> String {
    uuid::Uuid::new_v4().to_string()
}
