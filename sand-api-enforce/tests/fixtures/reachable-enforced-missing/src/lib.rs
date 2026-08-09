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
