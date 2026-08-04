use std::{sync::Arc, time::Duration, time::Instant};

use resource_space::{
    AddressCasePolicy, FolderId, InMemoryResourceSpace, ResourceMetadata, ResourceName,
    ResourceRootDescriptor, ResourceRootId, StoreId, VisibilityQuery,
};

const ENTRY_COUNT: usize = 2_048;
const REPEATED_READS: usize = 8_192;
const COPY_COUNT: usize = 512;

/// Raw timings from one deterministic in-memory Resource Space workload.
///
/// These observations are intended to catch unexpected algorithmic regression,
/// not to establish a machine-independent performance contract.
#[derive(Debug)]
pub struct ResourceSpaceBenchmark {
    entries: usize,
    repeated_reads: usize,
    copies: usize,
    listing_elapsed: Duration,
    read_elapsed: Duration,
    copy_elapsed: Duration,
    copied_entry_shares_bytes: bool,
    retained_bytes: usize,
}

impl ResourceSpaceBenchmark {
    pub const fn entries(&self) -> usize {
        self.entries
    }

    pub const fn repeated_reads(&self) -> usize {
        self.repeated_reads
    }

    pub const fn copies(&self) -> usize {
        self.copies
    }

    pub const fn listing_elapsed(&self) -> Duration {
        self.listing_elapsed
    }

    pub const fn read_elapsed(&self) -> Duration {
        self.read_elapsed
    }

    pub const fn copy_elapsed(&self) -> Duration {
        self.copy_elapsed
    }

    pub const fn copied_entry_shares_bytes(&self) -> bool {
        self.copied_entry_shares_bytes
    }

    pub const fn retained_bytes(&self) -> usize {
        self.retained_bytes
    }
}

pub fn run_resource_space_benchmark(
) -> Result<ResourceSpaceBenchmark, resource_space::ResourceSpaceError> {
    run_with_counts(ENTRY_COUNT, REPEATED_READS, COPY_COUNT)
}

fn run_with_counts(
    entry_count: usize,
    repeated_reads: usize,
    copy_count: usize,
) -> Result<ResourceSpaceBenchmark, resource_space::ResourceSpaceError> {
    let mut space =
        InMemoryResourceSpace::new(StoreId::from_u128(900), AddressCasePolicy::Sensitive);
    let root = FolderId::from_u128(901);
    let copies = FolderId::from_u128(902);
    space.create_root(
        ResourceRootDescriptor::new(ResourceRootId::from_u128(903), "benchmark"),
        root,
        ResourceMetadata::default(),
    )?;
    space.create_folder(
        copies,
        root,
        resource_name("copies"),
        ResourceMetadata::default(),
    )?;

    let shared_bytes: Arc<[u8]> = Arc::from(vec![0xAB; 1_024]);
    for index in 0..entry_count {
        space.insert_resource(
            root,
            resource_name(&format!("entry-{index:04}.bin")),
            Arc::clone(&shared_bytes),
            ResourceMetadata::default(),
        )?;
    }

    let listing_started = Instant::now();
    let listing = space.list_resources(root, VisibilityQuery::All)?;
    let listing_elapsed = listing_started.elapsed();
    assert_eq!(listing.len(), entry_count);

    let repeated_name = resource_name(&format!("entry-{:04}.bin", entry_count / 2));
    let read_started = Instant::now();
    for _ in 0..repeated_reads {
        let entry = space
            .resource(root, &repeated_name)?
            .expect("benchmark source entry remains available");
        assert_eq!(entry.byte_len(), shared_bytes.len());
    }
    let read_elapsed = read_started.elapsed();

    let source_name = resource_name("entry-0000.bin");
    let source = space
        .resource(root, &source_name)?
        .expect("benchmark source entry exists");
    let copy_started = Instant::now();
    let mut copied_entry_shares_bytes = true;
    for index in 0..copy_count {
        let copied = space.copy_resource(
            root,
            &source_name,
            copies,
            resource_name(&format!("copy-{index:04}.bin")),
        )?;
        copied_entry_shares_bytes &= Arc::ptr_eq(source.bytes(), copied.bytes());
        assert_ne!(source.key(), copied.key());
    }
    let copy_elapsed = copy_started.elapsed();

    Ok(ResourceSpaceBenchmark {
        entries: entry_count,
        repeated_reads,
        copies: copy_count,
        listing_elapsed,
        read_elapsed,
        copy_elapsed,
        copied_entry_shares_bytes,
        retained_bytes: space.summary().retained_bytes(),
    })
}

fn resource_name(value: &str) -> ResourceName {
    ResourceName::parse(value, AddressCasePolicy::Sensitive).expect("static benchmark name")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workload_reports_distinct_entries_with_shared_copy_bytes() {
        let observation = run_with_counts(32, 64, 8).expect("workload");

        assert_eq!(observation.entries(), 32);
        assert_eq!(observation.repeated_reads(), 64);
        assert_eq!(observation.copies(), 8);
        assert!(observation.copied_entry_shares_bytes());
        assert_eq!(observation.retained_bytes(), (32 + 8) * 1_024);
    }
}
