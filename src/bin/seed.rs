use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::env;
use std::fs;

async fn run_seeders(pool: &Pool<Postgres>) -> sqlx::Result<()> {
    let seeder_paths = vec![
        "seeders/20240115154908_users.sql",
        "seeders/20240115154929_organizations.sql",
        "seeders/20240115154942_locations.sql",
        "seeders/20240115154957_org_locations.sql",
        "seeders/20240115155009_org_staff.sql",
        "seeders/20240115155020_shifts.sql",
        "seeders/20240115155032_shift_org_staff.sql",
        "seeders/20240115155042_organization_invites.sql",
        "seeders/20240115155055_location_invites.sql"
    ];

    for path in seeder_paths {
        let sql = fs::read_to_string(path)?;
        sqlx::query(&sql).execute(pool).await?;
    }

    Ok(())
}
#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to connect to the database");

    if let Err(e) = run_seeders(&pool).await {
        eprintln!("Failed to run seeders: {:?}", e);
    }
}
