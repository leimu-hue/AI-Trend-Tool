fn main() {
    // Tell Cargo to recompile whenever any migration file changes.
    // sqlx::migrate!() embeds migration checksums at compile time;
    // without this, editing a .sql file after compilation causes
    // runtime VersionMismatch errors because Cargo won't recompile.
    println!("cargo:rerun-if-changed=docs/migrations");
}
