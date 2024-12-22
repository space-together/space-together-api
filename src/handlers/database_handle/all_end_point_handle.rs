use actix_web::{HttpResponse, Responder};

use crate::models::database_model::all_end_point_models::{EndpointCategoryModel, EndpointMolder};

/// Group endpoints into categories based on their path prefix.
pub async fn list_all_endpoints() -> impl Responder {
    let endpoints = vec![
        EndpointMolder {
            method: "GET".to_string(),
            path: "/api/v0.0.1/".to_string(),
        },
        EndpointMolder {
            method: "GET".to_string(),
            path: "/api/v0.0.1/users".to_string(),
        },
        EndpointMolder {
            method: "POST".to_string(),
            path: "/api/v0.0.1/users/role".to_string(),
        },
        EndpointMolder {
            method: "GET".to_string(),
            path: "/api/v0.0.1/users/role".to_string(),
        },
        EndpointMolder {
            method: "PUT".to_string(),
            path: "/api/v0.0.1/users/role/{id}".to_string(),
        },
        EndpointMolder {
            method: "DELETE".to_string(),
            path: "/api/v0.0.1/users/role/{id}".to_string(),
        },
        EndpointMolder {
            method: "GET".to_string(),
            path: "/api/v0.0.1/users/role/{name}".to_string(),
        },
        EndpointMolder {
            method: "POST".to_string(),
            path: "/api/v0.0.1/classes".to_string(),
        },
        EndpointMolder {
            method: "GET".to_string(),
            path: "/api/v0.0.1/classes".to_string(),
        },
        EndpointMolder {
            method: "POST".to_string(),
            path: "/api/v0.0.1/classes/class_groups".to_string(),
        },
        EndpointMolder {
            method: "GET".to_string(),
            path: "/api/v0.0.1/classes/class_groups".to_string(),
        },
        EndpointMolder {
            method: "POST".to_string(),
            path: "/api/v0.0.1/requests".to_string(),
        },
        EndpointMolder {
            method: "GET".to_string(),
            path: "/api/v0.0.1/requests".to_string(),
        },
        EndpointMolder {
            method: "GET".to_string(),
            path: "/api/v0.0.1/requests/{id}".to_string(),
        },
    ];

    let mut categories: Vec<EndpointCategoryModel> = vec![];

    for endpoint in endpoints {
        let prefix = endpoint
            .path
            .split('/')
            .nth(3) // Get the first meaningful segment after the base path
            .unwrap_or("root")
            .to_string();

        match categories.iter_mut().find(|c| c.name == prefix) {
            Some(category) => category.endpoints.push(endpoint),
            None => categories.push(EndpointCategoryModel {
                name: prefix,
                endpoints: vec![endpoint],
            }),
        }
    }

    HttpResponse::Ok().json(categories)
}
