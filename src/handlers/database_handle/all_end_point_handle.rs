use actix_web::{HttpResponse, Responder};

use crate::models::database_model::all_end_point_models::EndpointMolder;

/// Group endpoints into a single 'other' category.
pub async fn list_all_endpoints() -> impl Responder {
    let endpoints = vec![
        EndpointMolder {
            method: "GET".to_string(),
            path: "/".to_string(),
        },
        EndpointMolder {
            method: "GET".to_string(),
            path: "/users".to_string(),
        },
        EndpointMolder {
            method: "POST".to_string(),
            path: "/users/role".to_string(),
        },
        EndpointMolder {
            method: "GET".to_string(),
            path: "/users/role".to_string(),
        },
        EndpointMolder {
            method: "PUT".to_string(),
            path: "/users/rl/{id}".to_string(), // get user
        },
        EndpointMolder {
            method: "DELETE".to_string(),
            path: "/users/role/{id}".to_string(),
        },
        EndpointMolder {
            method: "GET".to_string(),
            path: "/users/role/{name}".to_string(),
        },
        EndpointMolder {
            method: "POST".to_string(),
            path: "/classes".to_string(),
        },
        EndpointMolder {
            method: "GET".to_string(),
            path: "/classes".to_string(),
        },
        EndpointMolder {
            method: "POST".to_string(),
            path: "/classes/class_groups".to_string(),
        },
        EndpointMolder {
            method: "GET".to_string(),
            path: "/classes/class_groups".to_string(),
        },
        EndpointMolder {
            method: "POST".to_string(),
            path: "/requests".to_string(),
        },
        EndpointMolder {
            method: "GET".to_string(),
            path: "/requests".to_string(),
        },
        EndpointMolder {
            method: "GET".to_string(),
            path: "/requests/{id}".to_string(),
        },
    ];

    HttpResponse::Ok().json(endpoints)
}
