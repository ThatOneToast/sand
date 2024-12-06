use crate::{components::ComponentBundle, selector::TargetSelector};



#[derive(Debug, Clone)]
pub struct Give {
    pub selector: TargetSelector,
    pub count: u8,
    pub item: String,
    pub components: Option<ComponentBundle>
}

impl Give {
    pub fn new(selector: TargetSelector, count: u8, item: String, components: Option<ComponentBundle>) -> Self {
        Self {
            selector,
            count,
            item,
            components
        }
    }
}

impl ToString for Give {
    fn to_string(&self) -> String {
        let mut s = String::from("/give ");
        s.push_str(self.selector.to_string().as_str());
        s.push_str(" ");
        s.push_str(&self.item);
        s.push('[');
        if let Some(components) = &self.components {
            s.push_str(components.to_string().as_str());
        };
        s.push(']');
        s.push_str(" ");
        s.push_str(self.count.to_string().as_str());
        s
    }
}