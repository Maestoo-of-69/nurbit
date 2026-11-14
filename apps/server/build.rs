// `sqlx::migrate!` embeds the SQL files at compile time. Tell Cargo to rebuild
// this crate whenever a migration is added or changed (recommended by SQLx).
fn main() {
    println!("cargo:rerun-if-changed=migrations");
}
