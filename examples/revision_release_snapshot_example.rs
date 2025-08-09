use swhid::{Revision, RevisionType, Release, ReleaseTargetType, Snapshot, SnapshotBranch, SnapshotTargetType, Person, Timestamp, TimestampWithTimezone};

fn main() {
    println!("=== Revision, Release, and Snapshot Example ===\n");

    // Create a Person for author/committer
    let author = Person::from_fullname("John Doe <john@example.com>").unwrap();
    let timestamp = Timestamp::new(1234567890, 0).unwrap();
    let date = TimestampWithTimezone::from_numeric_offset(timestamp, 0, false);

    // Create a Revision
    println!("--- Revision Example ---");
    let directory = [0u8; 20]; // This would normally be a real directory hash
    let revision = Revision::new(
        Some(b"Initial commit".to_vec()),
        Some(author.clone()),
        Some(author.clone()),
        Some(date.clone()),
        Some(date.clone()),
        RevisionType::Git,
        directory,
        false,
        None,
        vec![], // No parents for initial commit
        vec![],
    );

    println!("Revision SWHID: {}", revision.swhid());
    println!("Directory SWHID: {}", revision.directory_swhid());
    println!("Message: {}", String::from_utf8_lossy(revision.message().unwrap()));
    println!("Author: {}", revision.author().unwrap());
    println!("Date: {}", revision.date().unwrap());

    // Create a Release
    println!("\n--- Release Example ---");
    let release = Release::new(
        b"v1.0.0".to_vec(),
        Some(b"Release v1.0.0".to_vec()),
        Some(*revision.id()), // Target the revision we just created
        ReleaseTargetType::Revision,
        false,
        Some(author.clone()),
        Some(date.clone()),
        None,
    );

    println!("Release SWHID: {}", release.swhid());
    println!("Release name: {}", String::from_utf8_lossy(release.name()));
    println!("Release message: {}", String::from_utf8_lossy(release.message().unwrap()));
    println!("Target SWHID: {}", release.target_swhid().unwrap());

    // Create a Snapshot
    println!("\n--- Snapshot Example ---");
    let mut branches = std::collections::HashMap::new();
    
    // Add main branch pointing to the revision
    let main_branch = SnapshotBranch::new((*revision.id()).to_vec(), SnapshotTargetType::Revision);
    branches.insert(b"main".to_vec(), Some(main_branch));
    
    // Add a tag branch pointing to the release
    let tag_branch = SnapshotBranch::new((*release.id()).to_vec(), SnapshotTargetType::Release);
    branches.insert(b"refs/tags/v1.0.0".to_vec(), Some(tag_branch));
    
    // Add an alias (like HEAD)
    let alias_branch = SnapshotBranch::new((*revision.id()).to_vec(), SnapshotTargetType::Alias);
    branches.insert(b"HEAD".to_vec(), Some(alias_branch));

    let snapshot = Snapshot::new(branches);

    println!("Snapshot SWHID: {}", snapshot.swhid());
    println!("Number of branches: {}", snapshot.branches().len());
    
    // Access specific branches
    if let Some(main_branch) = snapshot.get_branch(b"main") {
        println!("Main branch target: {}", hex::encode(main_branch.target()));
        println!("Main branch type: {}", main_branch.target_type());
        if let Some(swhid) = main_branch.swhid() {
            println!("Main branch SWHID: {}", swhid);
        }
    }

    if let Some(head_branch) = snapshot.get_branch(b"HEAD") {
        println!("HEAD branch type: {}", head_branch.target_type());
        // Aliases don't have SWHIDs
        assert_eq!(head_branch.swhid(), None);
    }

    println!("\n--- SWHID Hierarchy ---");
    println!("Content -> Directory -> Revision -> Release -> Snapshot");
    println!("Each level builds upon the previous one:");
    println!("- Content: Individual files");
    println!("- Directory: Collections of files");
    println!("- Revision: Git commits with directory and metadata");
    println!("- Release: Tags pointing to revisions");
    println!("- Snapshot: Complete state of a repository at a point in time");
} 