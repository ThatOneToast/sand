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

pub mod version {
    pub struct VersionProfile;

    impl VersionProfile {
        pub fn uncontracted_capability(&self) -> bool {
            false
        }
    }
}

pub mod vfx {
    pub struct Vfx;

    impl Vfx {
        pub fn uncontracted_step(self) -> Self {
            self
        }
    }

    pub enum Visibility {
        UncontractedMode,
    }
}

pub mod advanced {
    pub fn try_export_components_json(_namespace: &str, _mc_version: &str) -> String {
        String::new()
    }

    pub fn uncontracted_hook() {}
}
