use axum::{extract::Path, Json, http::StatusCode, response::IntoResponse};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Organization {
    organization_name: String,
    id: String,
}

#[derive(Serialize, Deserialize)]
pub struct CreateOrganizationRequest {
    pub name: String,
    // Add other fields as necessary
}

#[derive(Serialize, Deserialize)]
pub struct Staff {
    id: String,
    user_id: String,
    organization_id: String,
    organization_title: String,
    // admin_created: Date
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

pub async fn get_organization_by_id(Path(org_id): Path<String>) -> impl IntoResponse {
    // Example list of organizations
    let organizations = vec![
        Organization {
            id: String::from("0001"),
            organization_name: String::from("V&P Wolfpack Security"),
        },
        // Add more organizations here
    ];

    // Find the organization with the provided ID
    match organizations.into_iter().find(|org| org.id == org_id) {
        Some(org) => Ok(Json(org)),
        None => Err((StatusCode::NOT_FOUND, "Organization not found".to_string())),
    }
}

pub async fn get_staff_user(Path((org_id, user_id)): Path<(String, String)>) -> impl IntoResponse {
    // Example: Retrieve a staff user based on org_id and user_id
    // Placeholder logic - replace with your actual logic to retrieve the staff user
    let staff_user = Staff {
        id: String::from("0001"),
        user_id: String::from("0001"),
        organization_id: String::from("0001"),
        organization_title: String::from("Sergeant"),
        // admin_created: chrono::Utc::now(),
    };

    // Placeholder condition to simulate staff user retrieval
    if staff_user.organization_id == org_id && staff_user.user_id == user_id {
        Ok(Json(staff_user))
    } else {
        Err((StatusCode::NOT_FOUND, "Staff User not found".to_string()))
    }
}

pub async fn create_organization(Json(payload): Json<CreateOrganizationRequest>) -> (StatusCode, Json<CreateOrganizationRequest>) {
    // Here, you would include logic to save the organization
    // For now, let's just return the received data

    (StatusCode::CREATED, Json(payload))
}


// pub async fn get_org_user_data(
//     Path((org_id, staff_id)): Path<(String, String)>,
//     db_pool: sqlx::Pool<sqlx::Postgres>
// ) -> impl IntoResponse {
//     // Query to find staff member within the organization
//     let query = sqlx::query_as!(OrgUser,
//         "SELECT staff.id, staff.first_name, staff.last_name, staff.title, staff.is_admin
//          FROM staff
//          INNER JOIN organizations ON staff.organization_id = organizations.id
//          WHERE organizations.id = $1 AND staff.id = $2",
//         org_id, staff_id
//     );

//     match query.fetch_one(&db_pool).await {
//         Ok(org_user_data) => (StatusCode::OK, Json(org_user_data)),
//         Err(_) => (StatusCode::NOT_FOUND, Json("Staff member not found in the organization")),
//     }
// }
