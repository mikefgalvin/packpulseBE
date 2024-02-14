use axum::extract::{State, self};
use axum::{http::StatusCode, response::IntoResponse, Json, extract::Path, Extension};
use serde::{Serialize, Deserialize};
use serde_json::{Value as JsonValue, json};
use sqlx::{PgPool, Row, Executor};
use std::sync::Arc;
use uuid::Uuid;
use chrono::{ Utc, DateTime};
use rrule::{RRuleSet, Tz};

use crate::auth_middleware::AuthenticatedUser;

#[derive(Serialize)]
struct OrgUserData {
    staff_id: Option<Uuid>,
    organization_id: Option<Uuid>,
    user_id: Option<Uuid>,
    title: Option<String>,
    admin_assigned_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
    first_name: String,
    last_name: String,
}

#[derive(Serialize)]
struct OrgShiftData {
    id: Option<Uuid>,
    start: Option<DateTime<Utc>>,
    end: Option<DateTime<Utc>>,
    title: Option<String>,
}


#[derive(Debug, Deserialize)]
struct ShiftPayload {
    organization_id: Uuid,
    location_id: Uuid,
    start_time: DateTime<Utc>, // Assuming startTime includes the timezone
    end_time: DateTime<Utc>, // Assuming endTime includes the timezone
    notes: String,
    rrule: String,
    assigned_staff: Vec<String>,
}

pub async fn get_organization(
    Path(organization_id): Path<String>,
    State(pool): State<Arc<PgPool>>,
    Extension(auth_user): Extension<AuthenticatedUser>,
) -> impl IntoResponse {
    let user_id = auth_user.id;
    
    let result = sqlx::query(
        "SELECT jsonb_build_object(
            'id', o.id,
            'name', o.name,
            'created_at', o.created_at,
            'user', (
                SELECT jsonb_build_object(
                    'id', u.id,
                    'first_name', u.first_name,
                    'last_name', u.last_name,
                    'title', os2.title,
            	    'admin_assigned_at', os2.admin_assigned_at
                )
                FROM users u
                join org_staff os2 on os2.user_id = u.id and os2.organization_id = o.id
                where u.id = $2::uuid
            ),
            'locations', (
                SELECT json_agg(jsonb_build_object(
                    'id', l.id,
                    'name', l.name
                ))
                FROM locations l 
                JOIN org_locations ol ON ol.location_id = l.id
                WHERE ol.organization_id = o.id
            ),
            'staff', (
                SELECT json_agg(jsonb_build_object(
                    'id', os.id,
                    'title', os.title,
                    'first_name', u.first_name,
                    'last_name', u.last_name
                ))
                FROM org_staff os  
                JOIN users u ON u.id = os.user_id
                WHERE os.organization_id = o.id
            )
        ) AS org_data
        FROM organizations o 
        WHERE o.id = $1::uuid"
    )
    .bind(organization_id)
    .bind(user_id)
    .fetch_one(&*pool)
    .await;

    match result {
        Ok(row) => {
            let org_data: JsonValue = row.get("org_data");
            (StatusCode::OK, Json(org_data))
        },
        Err(e) => {
            eprintln!("Failed to execute query: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Internal server error"})))
        }
    }
}

pub async fn get_org_user(
    Path((organization_id, user_id)): Path<(Uuid, Uuid)>,
    Extension(pool): Extension<Arc<PgPool>>,
) -> impl IntoResponse {
    let result = sqlx::query_as!(
        OrgUserData,
        "SELECT os.id as staff_id, os.organization_id, os.user_id, os.title, 
                os.admin_assigned_at, os.created_at, u.first_name, u.last_name
         FROM org_staff os 
         JOIN users u ON u.id = os.user_id
         WHERE os.organization_id = $1 AND u.id = $2",
         organization_id, user_id
    )
    .fetch_one(&*pool)
    .await;

    match result {
        Ok(org_user_data) => {
            (StatusCode::OK, Json(org_user_data)).into_response()
        },
        Err(e) => {
            eprintln!("Failed to execute query: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR, 
                Json(json!({"error": "Internal server error"}))
            ).into_response()
        }
    }
}

pub async fn get_org_shifts(
    Path(organization_id): Path<String>,
    State(pool): State<Arc<PgPool>>,
    Extension(auth_user): Extension<AuthenticatedUser>,
) -> impl IntoResponse {
    let user_id = auth_user.id;
    
    let result = sqlx::query(
    "SELECT 
            s.id,
            s.start_timestamp as start, 
            s.end_timestamp as end, 
            l.name as title,
            l.address,
            l.description,
            s.rrule,
            s.notes,
            CASE 
                WHEN COUNT(os.id) > 0 THEN 'blue'
                ELSE 'red'
            END as color,
            s.extended_props,
            json_agg(
                jsonb_build_object(
                    'staff_id', os.id,
                    'user_id', os.user_id,
                    'first_name', u.first_name,
                    'last_name', u.last_name,
                    'org_title', os.title
                )
            ) FILTER (WHERE os.id IS NOT NULL) as assigned
        FROM shifts s
        JOIN locations l ON l.id = s.location_id
        LEFT JOIN shift_org_staff sos ON sos.shift_id = s.id
        LEFT JOIN org_staff os ON os.id = sos.org_staff_id
        LEFT JOIN users u on u.id = os.user_id
        WHERE s.organization_id = $1::uuid
        AND EXISTS (
            SELECT 1
            FROM org_staff admin_os
            WHERE admin_os.user_id = $2::uuid
            AND admin_os.admin_assigned_at IS NOT NULL
            AND admin_os.organization_id = s.organization_id
        )
        GROUP BY s.id, l.id"
    )
    .bind(organization_id)
    .bind(user_id)
    .fetch_all(&*pool)
    .await;

    match result {
        Ok(rows) => {
            let shifts_data: Vec<serde_json::Value> = rows.into_iter().map(|row| {
                json!({
                    "id": row.try_get::<DateTime<Utc>, _>("id").unwrap_or_default(),
                    "start": row.try_get::<DateTime<Utc>, _>("start").unwrap_or_default(),
                    "end": row.try_get::<DateTime<Utc>, _>("end").unwrap_or_default(),
                    "title": row.try_get::<String, _>("title").unwrap_or_default(),
                    "address": row.try_get::<String, _>("address").unwrap_or_default(),
                    "description": row.try_get::<String, _>("description").unwrap_or_default(),
                    "rrule": row.try_get::<String, _>("rrule").unwrap_or_default(),
                    "notes": row.try_get::<String, _>("notes").unwrap_or_default(),
                    "color":row.try_get::<String, _>("color").unwrap_or_default(),
                    "assigned": row.try_get::<serde_json::Value, _>("assigned").unwrap_or_default(),
                    "extended_props": row.try_get::<serde_json::Value, _>("extended_props").unwrap_or_default(),
                })
            }).collect();
    
            (StatusCode::OK, Json(shifts_data))
        },
        Err(e) => {
            eprintln!("Failed to execute query: {:?}", e);
            // Return an error in the same format (Vec<JsonValue>)
            let error_response = vec![json!({"error": "Internal server error"})];
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        }
    
    }
}

pub async fn get_user_org_shifts(
    Path(organization_id): Path<String>,
    State(pool): State<Arc<PgPool>>,
    Extension(auth_user): Extension<AuthenticatedUser>,
) -> impl IntoResponse {
    let user_id = auth_user.id;
    
    let result = sqlx::query(
    "SELECT 
            s.start_timestamp as start, 
            s.end_timestamp as end, 
            l.name as title,
            l.address,
            l.description,
            s.rrule,
            s.notes,
            s.extended_props
        FROM shifts s
        JOIN shift_org_staff sos ON sos.shift_id = s.id
        JOIN org_staff os ON os.id = sos.org_staff_id 
        JOIN locations l ON l.id = s.location_id
        WHERE os.user_id = $2::uuid AND s.organization_id = $1::uuid"
    )
    .bind(organization_id)
    .bind(user_id)
    .fetch_all(&*pool)
    .await;

    match result {
        Ok(rows) => {
            let shifts_data: Vec<serde_json::Value> = rows.into_iter().map(|row| {
                json!({
                    "start": row.try_get::<DateTime<Utc>, _>("start").unwrap_or_default(),
                    "end": row.try_get::<DateTime<Utc>, _>("end").unwrap_or_default(),
                    "title": row.try_get::<String, _>("title").unwrap_or_default(),
                    "address": row.try_get::<String, _>("address").unwrap_or_default(),
                    "description": row.try_get::<String, _>("description").unwrap_or_default(),
                    "rrule": row.try_get::<String, _>("rrule").unwrap_or_default(),
                    "notes": row.try_get::<String, _>("notes").unwrap_or_default(),
                    "extended_props": row.try_get::<serde_json::Value, _>("extended_props").unwrap_or_default(),
                })
            }).collect();
    
            (StatusCode::OK, Json(shifts_data))
        },
        Err(e) => {
            eprintln!("Failed to execute query: {:?}", e);
            // Return an error in the same format (Vec<JsonValue>)
            let error_response = vec![json!({"error": "Internal server error"})];
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        }
    
    }
}

pub async fn create_shift(
    Path(organization_id): Path<String>,
    State(pool): State<Arc<PgPool>>,
    extract::Json(payload): extract::Json<ShiftPayload>,
) -> impl IntoResponse {
    // Parse the RRuleSet from the payload's rrule string
    let rrule_set: RRuleSet = payload.rrule.parse().unwrap_or_else(|_| panic!("Failed to parse RRule string"));

    // Generate occurrences with a limit to avoid infinite loops
    let limit = 100;
    let occurrences = rrule_set.all(limit).dates;

    for &occurrence in &occurrences {
        // Directly use organization_id and location_id as they are already Uuids
        let shift_result = sqlx::query!(
            "INSERT INTO shifts (organization_id, location_id, start_timestamp, end_timestamp, notes) VALUES ($1, $2, $3, $4, $5) RETURNING id",
            payload.organization_id,
            payload.location_id,
            occurrence,
            occurrence + (payload.end_time - payload.start_time), // This line may need adjustment based on how you calculate the end time for each occurrence
            payload.notes
        )
        .fetch_one(&*pool)
        .await;
    
        match shift_result {
            Ok(record) => {
                let shift_id = record.id; // Assuming `id` is the name of the column in your RETURNING clause
    
                // Use `shift_id` for your staff assignments, converting assigned_staff member IDs from String to Uuid
                for staff_id_str in &payload.assigned_staff {
                    let staff_id = staff_id_str.parse::<Uuid>()
                        .expect("Failed to parse staff_id into Uuid");
                    
                    let _ = sqlx::query!(
                        "INSERT INTO shift_org_staff (shift_id, org_staff_id) VALUES ($1, $2)",
                        shift_id,
                        staff_id
                    )
                    .execute(&*pool)
                    .await
                    .map_err(|e| {
                        eprintln!("Failed to assign staff to shift: {:?}", e);
                        return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to assign staff: {}", e)).into_response();
                    })?;
                }
            },
            Err(e) => {
                eprintln!("Failed to insert shift: {:?}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create shift: {}", e)).into_response();
            },
        }
    }
    
    // Optionally handle assigned staff
    // For each `occurrence`, you might need to create associations in `org_staff_shifts` table

    (StatusCode::OK, format!("Successfully created {} shifts.", occurrences.len())).into_response()
}