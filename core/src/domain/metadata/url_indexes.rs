nest! {
    #[derive(Debug, Clone, PartialEq, Eq, Builder, Serialize, Deserialize)]*
    pub struct UrlIndexes {
        pub values: Vec<
            pub struct UrlIndex {
                /// The fields to index on, e.g. ["long", "userId"] for a compound index
                /// This works for both SQL columns and NoSQL fields
                pub keys: Vec<String>,
                /// Whether this index enforces uniqueness constraints
                /// For SQL databases, this maps to UNIQUE constraint
                pub is_unique: bool,
                /// Whether this index skips null/missing values
                /// For SQL databases, this maps to WHERE column IS NOT NULL
                pub is_sparse: bool,
            },
        >,
    }
}

impl Default for UrlIndexes {
    fn default() -> Self {
        Self {
            values: vec![
                UrlIndex::builder()
                    .keys(vec!["user_id", "long"])
                    .is_unique(true)
                    .is_sparse(false)
                    .build(),
                UrlIndex::builder()
                    .keys(vec!["short"])
                    .is_unique(true)
                    .is_sparse(false)
                    .build(),
                UrlIndex::builder()
                    .keys(vec!["alias"])
                    .is_unique(true)
                    .is_sparse(false)
                    .build(),
            ],
        }
    }
}

impl UrlIndexes {
    pub fn new(values: Vec<UrlIndex>) -> Self {
        Self { values }
    }
}
