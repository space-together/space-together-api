#[derive(Debug)]
pub struct CollectionStats {
    pub name: String,
    pub document_count: u64,
    pub size_bytes: String,
}

#[derive(Debug)]
pub struct DatabaseStats {
    pub total_documents: u64,
    pub total_size_bytes: String,
    pub collections: Vec<CollectionStats>,
}
