use std::sync::Arc;
use tonic::{Request, Response, Status};
use crate::api_server::server_state::ServerState;
use crate::database::{models, DbBasicQuery};
use crate::auth::AuthManager;
use lib_clapshot_grpc::proto::org::{
    hema_auth_service_server::HemaAuthService,
    LoginRequest, LoginResponse,
    ValidateSessionRequest, ValidateSessionResponse,
    CreateUserByAdminRequest
};
use lib_clapshot_grpc::proto::{self};

pub struct HemaAuthServiceImpl {
    server: ServerState,
}

impl HemaAuthServiceImpl {
    pub fn new(server: ServerState) -> Self {
        Self { server }
    }
}

#[tonic::async_trait]
impl HemaAuthService for HemaAuthServiceImpl {
    async fn login(&self, req: Request<LoginRequest>) -> Result<Response<LoginResponse>, Status> {
        let req = req.into_inner();
        let mut conn = self.server.db.conn().map_err(|e| Status::internal(e.to_string()))?;

        // Find user in DB
        let user = models::User::get(&mut conn, &req.username)
            .map_err(|_| Status::unauthenticated("Invalid username or password"))?;

        let password_hash = user.password_hash.as_ref()
            .ok_or_else(|| Status::unauthenticated("User has no password set"))?;

        // Verify password
        if !AuthManager::verify_password(&req.password, password_hash) {
            return Err(Status::unauthenticated("Invalid username or password"));
        }

        // Generate token
        let token = self.server.auth_manager.generate_token(&user.id, user.is_admin)
            .map_err(|e| Status::internal(format!("Token generation failed: {}", e)))?;

        Ok(Response::new(LoginResponse {
            token,
            user: Some(proto::UserInfo {
                id: user.id,
                name: user.name,
            }),
        }))
    }

    async fn validate_session(&self, req: Request<ValidateSessionRequest>) -> Result<Response<ValidateSessionResponse>, Status> {
        let req = req.into_inner();
        
        let claims = self.server.auth_manager.validate_token(&req.token)
            .map_err(|e| Status::unauthenticated(format!("Invalid token: {}", e)))?;

        let mut conn = self.server.db.conn().map_err(|e| Status::internal(e.to_string()))?;
        let user = models::User::get(&mut conn, &claims.sub)
            .map_err(|_| Status::unauthenticated("User not found"))?;

        Ok(Response::new(ValidateSessionResponse {
            user: Some(proto::UserInfo {
                id: user.id,
                name: user.name,
            }),
            is_admin: user.is_admin,
        }))
    }

    async fn create_user_by_admin(&self, req: Request<CreateUserByAdminRequest>) -> Result<Response<proto::UserInfo>, Status> {
        // TODO: Add actual admin check from request metadata/interceptor
        // For now, we assume this is called by an authorized admin context
        
        let req = req.into_inner();
        let mut conn = self.server.db.conn().map_err(|e| Status::internal(e.to_string()))?;

        let password_hash = AuthManager::hash_password(&req.password)
            .map_err(|e| Status::internal(format!("Hashing failed: {}", e)))?;

        let new_user = models::UserInsert {
            id: req.username.clone(),
            name: req.name.unwrap_or(req.username.clone()),
            password_hash: Some(password_hash),
            is_admin: req.is_admin,
            language: Some("ru".to_string()),
            color: None,
            avatar: None,
        };

        let user = models::User::insert(&mut conn, &new_user)
            .map_err(|e| Status::internal(format!("Failed to create user: {}", e)))?;

        Ok(Response::new(proto::UserInfo {
            id: user.id,
            name: user.name,
        }))
    }
}
