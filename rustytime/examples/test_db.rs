use diesel::prelude::*;
use rustytime::db::create_pool;
use rustytime::models::*;
use rustytime::schema::heartbeats;
use chrono::Utc;
use ipnetwork::IpNetwork;

fn main() {
    // test database connection
    let mut connection = create_pool().get().expect("Failed to get database connection");
    println!("✅ Database connection established successfully!");

    // use a sample user ID
    let user_id: i32 = 1;

    // create the user if it doesn't exist
    diesel::sql_query("INSERT INTO users (id) VALUES ($1) ON CONFLICT (id) DO NOTHING")
        .bind::<diesel::sql_types::Integer, _>(1)
        .execute(&mut connection)
        .expect("Failed to insert user");

    // create a sample heartbeat
    let ip_network = IpNetwork::from("127.0.0.1".parse::<std::net::IpAddr>().unwrap());
    let new_heartbeat = NewHeartbeat::new(Utc::now(), user_id, "test_file.rs".to_string(), "file".to_string(), ip_network)
        .with_project("test_project".to_string())
        .with_language("rust".to_string())
        .with_lines(100)
        .with_editor("vscode".to_string());

    // insert the heartbeat
    match diesel::insert_into(heartbeats::table)
        .values(&new_heartbeat)
        .get_result::<Heartbeat>(&mut connection)
    {
        Ok(heartbeat) => {
            println!("✅ Heartbeat inserted successfully!");
            println!("📊 Heartbeat data: {:?}", heartbeat);
        }
        Err(e) => {
            eprintln!("❌ Error inserting heartbeat: {}", e);
        }
    }

    // query heartbeats for this user
    match heartbeats::table
        .filter(heartbeats::user_id.eq(user_id))
        .select(Heartbeat::as_select())
        .load(&mut connection)
    {
        Ok(user_heartbeats) => {
            println!(
                "📋 Found {} heartbeats for user {}",
                user_heartbeats.len(),
                user_id
            );
            for heartbeat in user_heartbeats {
                println!("  - {} ({})", heartbeat.entity, heartbeat.type_);
            }
        }
        Err(e) => {
            eprintln!("❌ Error querying heartbeats: {}", e);
        }
    }
}
