use crate::selector::TargetSelector;


#[derive(Debug, Clone)]
pub struct Clear(pub Option<TargetSelector>);

impl ToString for Clear {
    fn to_string(&self) -> String {
        let str = String::from("/clear");
        if let Some(target) = &self.0 {
            format!("{} {}", str, target.to_string())
        } else {
            str
        }
    }
}