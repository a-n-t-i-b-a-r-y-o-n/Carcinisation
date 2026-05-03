use std::convert::Infallible;

use tonic::body::Body;
use tonic::codegen::http::{Request, Response};
use tonic::service::{AxumBody, Routes};
use tonic::transport::Server;
use tonic::transport::server::Router;
use tower_service::Service;

/// Thing with a name. Essentially [`tonic::server::NamedService`] but without the `const`.
pub trait Named {
    fn name(&self) -> &str;
}

/// Extension trait for [`Server`] allowing runtime route creation
pub trait DynamicRoutesExt {

    /// Add dynamic services at runtime
    fn add_dynamic_routes<S>(&mut self, services: Vec<S>) -> Router
    where
        S: Service<Request<AxumBody>, Response = Response<Body>, Error = Infallible>
        + Named
        + Clone
        + Send
        + Sync
        + 'static,
        S::Future: Send + 'static;
}

impl DynamicRoutesExt for Server {
    fn add_dynamic_routes<S>(&mut self, services: Vec<S>) -> Router
    where
        S: Service<Request<AxumBody>, Response = Response<Body>, Error = Infallible>
        + Named
        + Clone
        + Send
        + Sync
        + 'static,
        S::Future: Send + 'static
    {
        // Build default routes
        let mut routes = Routes::default();

        // Get inner router
        let mut router = routes.axum_router_mut().clone();

        // Add dynamic service routes
        for service in services {

            router = router.route_service(
                &format!("/{}/{{*rest}}", service.name()),
                service
            );

        }

        // Mutate inner router
        *routes.axum_router_mut() = router;

        // Return updated self
        self.add_routes(routes)
    }
}