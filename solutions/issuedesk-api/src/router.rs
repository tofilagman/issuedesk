use axum::{
    extract::DefaultBodyLimit,
    middleware,
    routing::{delete, get, patch, post},
    Router,
};
use tower_http::{services::ServeDir, trace::TraceLayer};

use crate::{auth::middleware::require_auth, handlers, state::AppState};

pub fn build(state: AppState) -> Router {
    let api = Router::new()
        // dashboards
        .route("/stats", get(handlers::stats::system))
        .route("/projects/{id}/stats", get(handlers::stats::project))
        // users
        .route("/users/me", get(handlers::users::me))
        .route("/users", get(handlers::users::list).post(handlers::users::create))
        .route("/users/{id}", patch(handlers::users::update))
        .route("/users/{id}/password", post(handlers::users::change_password))
        // projects
        .route("/projects", get(handlers::projects::list).post(handlers::projects::create))
        .route(
            "/projects/{id}",
            get(handlers::projects::get)
                .patch(handlers::projects::update)
                .delete(handlers::projects::delete),
        )
        // members
        .route(
            "/projects/{id}/members",
            get(handlers::members::list).post(handlers::members::add),
        )
        .route("/projects/{id}/members/{userId}", delete(handlers::members::remove))
        // issues (project-scoped)
        .route(
            "/projects/{id}/issues",
            get(handlers::issues::list).post(handlers::issues::create),
        )
        .route("/projects/{id}/issues/{number}", get(handlers::issues::get_by_number))
        // labels (project-scoped)
        .route(
            "/projects/{id}/labels",
            get(handlers::labels::list).post(handlers::labels::create),
        )
        // issues (by id)
        .route(
            "/issues/{id}",
            get(handlers::issues::get)
                .patch(handlers::issues::update)
                .delete(handlers::issues::delete),
        )
        .route(
            "/issues/{id}/comments",
            get(handlers::comments::list).post(handlers::comments::create),
        )
        .route("/issues/{id}/labels", post(handlers::labels::attach))
        .route("/issues/{id}/labels/{labelId}", delete(handlers::labels::detach))
        .route(
            "/issues/{id}/attachments",
            get(handlers::attachments::list).post(handlers::attachments::upload),
        )
        .route("/issues/{id}/activity", get(handlers::activity::list))
        .route(
            "/issues/{id}/links",
            get(handlers::links::list).post(handlers::links::create),
        )
        .route("/issue-links/{id}", delete(handlers::links::delete))
        // comments / labels / attachments (by id)
        .route(
            "/comments/{id}",
            patch(handlers::comments::update).delete(handlers::comments::delete),
        )
        .route("/labels/{id}", delete(handlers::labels::delete))
        .route(
            "/attachments/{id}",
            get(handlers::attachments::download).delete(handlers::attachments::delete),
        )
        // Allow large uploads (size is enforced while streaming in the handler).
        .layer(DefaultBodyLimit::max(state.config.max_upload_bytes + 1024 * 1024))
        // Guard the whole /api surface.
        .route_layer(middleware::from_fn_with_state(state.clone(), require_auth));

    let open = Router::new()
        .route("/auth/signIn", post(handlers::auth::sign_in))
        .route("/health", get(|| async { "ok" }));

    // SPA: serve the built SvelteKit assets, falling back to index.html for
    // client-side (hash) routing.
    let static_dir = state.config.static_dir.clone();
    let spa = ServeDir::new(&static_dir)
        .append_index_html_on_directories(true)
        .fallback(tower_http::services::ServeFile::new(format!(
            "{static_dir}/index.html"
        )));

    Router::new()
        .merge(open)
        .nest("/api", api)
        .fallback_service(spa)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
