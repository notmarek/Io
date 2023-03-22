use crate::{api, models};
use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        api::user::register,
        api::user::login,
        api::user::user_info,
        api::user::user_list,
    ),
    components(
        schemas(
            crate::ErrorResponse,
            api::user::Tokens,
            api::user::UserRequest,
            api::user::RegisterRequest,
            models::user::User,
        )
    ),
    modifiers(&SecurityAddon),
    )
]
pub struct ApiDoc;

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
