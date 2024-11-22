use crate::selector::TargetSelector;

#[derive(Debug, Clone)]
pub struct Kill(pub TargetSelector);

impl ToString for Kill {
    fn to_string(&self) -> String {
        format!("/kill {}", self.0.to_string())
    }
}