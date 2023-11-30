use crate::api;
use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi, ToSchema,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        api::user::register,
        api::user::login,
        api::user::user_info,
        api::user::user_list,
        api::user::edit_user,
        api::library::libraries,
        api::library::library,
        api::library::create_library,
        api::library::scan_library,
        api::library::delete_library,
        api::info::info,
        api::file::file,
        api::search::search,
    ),
    components(
        schemas(
            crate::ErrorResponse,
            crate::Response<String>,
            api::user::Tokens,
            api::user::UserRequest,
            api::user::RegisterRequest,
            api::user::EditUser,
            api::library::Lib,
            api::info::Info,
            entity::user::Model,
            entity::library::Model,
            T,
        )
    ),
    modifiers(&SecurityAddon),
    )
]
pub struct ApiDoc;

/// Filler schema to get rid of errors cause by type templates
#[derive(ToSchema)]
struct T;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        openapi.components.as_mut().unwrap().add_security_scheme(
            "token",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        )
    }
}
