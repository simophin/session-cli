use include_dir::{include_dir, Dir};
use rusqlite_migration::{Migrations, MigrationsBuilder};

static MIGRATIONS_DIR: Dir = include_dir!("migrations");

pub fn create_migrations() -> Migrations<'static> {
    MigrationsBuilder::from_directory(&MIGRATIONS_DIR)
        .expect("To build migrations")
        .finalize()
}
