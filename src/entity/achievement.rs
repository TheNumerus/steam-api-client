use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Achievement {
    pub apiname: String,
    pub achieved: bool,
    pub unlocktime: i64,
    pub name: String,
    pub description: Option<String>,
    pub percent: f32,
    pub icon_achieved: String,
    pub icon: String,
}
