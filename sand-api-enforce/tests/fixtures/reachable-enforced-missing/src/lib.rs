pub mod predicate {
    pub struct Builder {
        pub uncontracted_field: bool,
    }

    impl Builder {
        #[doc(hidden)]
        pub fn uncontracted_method(&self) {}
    }

    pub enum Choice {
        UncontractedVariant,
    }
}

pub mod execute_when {
    pub struct WhenBuilder;

    impl WhenBuilder {
        pub fn uncontracted_branch(self) {}
    }
}

pub mod condition {
    pub struct Condition;

    impl Condition {
        pub fn uncontracted_leaf(self) {}
    }
}

pub mod resource_ref {
    pub struct DialogId;

    impl DialogId {
        pub fn uncontracted_local(path: &str) -> Self {
            let _ = path;
            Self
        }
    }
}
