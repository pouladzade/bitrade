#[derive(Debug, Clone, Default)]
pub struct Pagination {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub order_by: Option<String>, // Allow ordering by different fields
    pub order_direction: Option<String>, // "asc" or "desc"
}

impl Pagination {
    pub fn new() -> Self {
        Self {
            limit: Some(100), // Default limit
            offset: Some(0),  // Default offset
            order_by: Some("created_at".to_string()),
            order_direction: Some("desc".to_string()),
        }
    }
}

pub struct Paginated<T> {
    pub items: Vec<T>,
    pub total_count: i64,
    pub next_offset: Option<i64>,
    pub has_more: bool,
}
