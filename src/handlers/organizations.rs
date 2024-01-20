use axum::{http::StatusCode, response::IntoResponse, Json, extract::Path, Extension};
use serde::{Serialize, Deserialize};
use serde_json::{Value as JsonValue, json};
use sqlx::{PgPool, Row};
use std::sync::Arc;
use uuid::Uuid;
use chrono::DateTime;
use chrono::Utc;

pub async fn get_organization(
    Path(organization_id): Path<String>,
    Extension(pool): Extension<Arc<PgPool>>,
) -> impl IntoResponse {
    let result = sqlx::query(
        "SELECT jsonb_build_object(
            'id', o.id,
            'name', o.name,
            'created_at', o.created_at,
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

#[derive(Serialize)]
struct OrgShiftData {
    id: Option<Uuid>,
    start: Option<DateTime<Utc>>,
    end: Option<DateTime<Utc>>,
    title: Option<String>,
}

pub async fn get_org_shifts(
    Path(organization_id): Path<Uuid>,
    Extension(pool): Extension<Arc<PgPool>>,
) -> impl IntoResponse {
    let result = sqlx::query_as!(
        OrgShiftData,
        "select s.id, s.start_timestamp as start, s.end_timestamp as end, s.timezone as title
        from shifts s 
        join shift_org_staff sos on sos.shift_id = s.id 
        join org_staff os on os.id = sos.org_staff_id 
        where s.organization_id = $1",
        organization_id
    )
    .fetch_one(&*pool)
    .await;

    match result {
        Ok(org_shift_data) => {
            (StatusCode::OK, Json(org_shift_data)).into_response()
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

