use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::schema::leaderboards;

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = leaderboards)]
pub struct Leaderboard {
    pub id: i32,
    pub user_id: i32,
    pub period_type: String,
    pub period_date: NaiveDate,
    pub total_seconds: i64,
    pub rank: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = leaderboards)]
pub struct NewLeaderboard {
    pub user_id: i32,
    pub period_type: String,
    pub period_date: NaiveDate,
    pub total_seconds: i64,
    pub rank: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct LeaderboardEntry {
    pub user_id: i32,
    pub user_name: String,
    pub avatar_url: String,
    pub total_seconds: i64,
    pub rank: i32,
}

impl Leaderboard {
    pub fn get_by_period(
        conn: &mut PgConnection,
        period_type: &str,
        period_date: NaiveDate,
    ) -> QueryResult<Vec<Leaderboard>> {
        leaderboards::table
            .filter(leaderboards::period_type.eq(period_type))
            .filter(leaderboards::period_date.eq(period_date))
            .order(leaderboards::rank.asc())
            .load::<Leaderboard>(conn)
    }

    pub fn upsert_batch(
        conn: &mut PgConnection,
        entries: Vec<NewLeaderboard>,
    ) -> QueryResult<usize> {
        if entries.is_empty() {
            return Ok(0);
        }

        diesel::insert_into(leaderboards::table)
            .values(&entries)
            .on_conflict((
                leaderboards::user_id,
                leaderboards::period_type,
                leaderboards::period_date,
            ))
            .do_update()
            .set((
                leaderboards::total_seconds
                    .eq(diesel::upsert::excluded(leaderboards::total_seconds)),
                leaderboards::rank.eq(diesel::upsert::excluded(leaderboards::rank)),
            ))
            .execute(conn)
    }

    pub fn delete_old_daily(conn: &mut PgConnection, cutoff_date: NaiveDate) -> QueryResult<usize> {
        diesel::delete(
            leaderboards::table
                .filter(leaderboards::period_type.eq("daily"))
                .filter(leaderboards::period_date.lt(cutoff_date)),
        )
        .execute(conn)
    }

    pub fn delete_old_weekly(
        conn: &mut PgConnection,
        cutoff_date: NaiveDate,
    ) -> QueryResult<usize> {
        diesel::delete(
            leaderboards::table
                .filter(leaderboards::period_type.eq("weekly"))
                .filter(leaderboards::period_date.lt(cutoff_date)),
        )
        .execute(conn)
    }
}
