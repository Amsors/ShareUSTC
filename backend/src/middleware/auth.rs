use actix_web::{
    body::MessageBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    error::ErrorUnauthorized,
    http::{header, Method},
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use std::{
    future::{ready, Ready},
    rc::Rc,
    task::{Context, Poll},
};

use crate::models::CurrentUser;
use crate::utils::{extract_current_user, verify_token};

/// Cookie 名称常量
const ACCESS_TOKEN_COOKIE: &str = "access_token";

/// 公开路径规则
#[derive(Clone)]
pub struct PublicPathRule {
    pub path_prefix: String,
    pub methods: Vec<Method>,
    pub exclude_paths: Vec<String>,
}

impl PublicPathRule {
    /// 创建允许所有方法的公开路径规则
    pub fn all_methods(path: &str) -> Self {
        Self {
            path_prefix: path.to_string(),
            methods: vec![],
            exclude_paths: vec![],
        }
    }

    /// 创建只允许特定方法的公开路径规则
    pub fn with_methods(path: &str, methods: Vec<Method>) -> Self {
        Self {
            path_prefix: path.to_string(),
            methods,
            exclude_paths: vec![],
        }
    }

    /// 设置排除的路径
    pub fn exclude(mut self, paths: Vec<&str>) -> Self {
        self.exclude_paths = paths.into_iter().map(|p| p.to_string()).collect();
        self
    }

    /// 检查是否匹配
    pub fn matches(&self, path: &str, method: &Method) -> bool {
        // 首先检查是否在排除列表中
        if self.exclude_paths.iter().any(|p| path.starts_with(p)) {
            return false;
        }

        if !path.starts_with(&self.path_prefix) {
            return false;
        }
        // 如果没有指定方法，则允许所有方法
        if self.methods.is_empty() {
            return true;
        }
        self.methods.contains(method)
    }
}

/// JWT认证中间件
pub struct JwtAuth {
    jwt_secret: String,
    public_paths: Vec<PublicPathRule>,
}

impl JwtAuth {
    pub fn new(jwt_secret: String) -> Self {
        Self {
            jwt_secret,
            public_paths: Vec::new(),
        }
    }

    /// 添加公开路径（不需要认证）- 向后兼容（预留接口）
    #[allow(dead_code)]
    pub fn with_public_paths(mut self, paths: Vec<String>) -> Self {
        self.public_paths = paths
            .into_iter()
            .map(|p| PublicPathRule::all_methods(&p))
            .collect();
        self
    }

    /// 添加带方法的公开路径规则
    pub fn with_public_rules(mut self, rules: Vec<PublicPathRule>) -> Self {
        self.public_paths = rules;
        self
    }

    /// 检查路径是否是公开路径（预留接口）
    #[allow(dead_code)]
    fn is_public_path(&self, path: &str, method: &Method) -> bool {
        self.public_paths.iter().any(|rule| rule.matches(path, method))
    }

}

impl<S, B> Transform<S, ServiceRequest> for JwtAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = JwtAuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtAuthMiddleware {
            service: Rc::new(service),
            jwt_secret: self.jwt_secret.clone(),
            public_paths: self.public_paths.clone(),
        }))
    }
}

pub struct JwtAuthMiddleware<S> {
    service: Rc<S>,
    jwt_secret: String,
    public_paths: Vec<PublicPathRule>,
}

impl<S, B> Service<ServiceRequest> for JwtAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let jwt_secret = self.jwt_secret.clone();
        let public_paths = self.public_paths.clone();

        Box::pin(async move {
            let path = req.path().to_string();
            let method = req.method().clone();

            // 检查是否是公开路径
            let is_public = public_paths.iter().any(|rule| rule.matches(&path, &method));

            // 从请求中提取 Token（从 Authorization 头或 Cookie）
            let token = Self::extract_token_from_request(&req);

            // 如果不是公开路径，必须有认证信息
            if !is_public && token.is_none() {
                log::debug!("非公开路径缺少认证信息: {} {}", method, path);
                return Err(ErrorUnauthorized("缺少认证信息"));
            }

            // 如果有认证信息，尝试验证
            if let Some(token) = token {
                // 验证Token
                match verify_token(&token, &jwt_secret, Some("access")) {
                    Ok(claims) => {
                        match extract_current_user(claims) {
                            Ok(current_user) => {
                                log::debug!("用户认证成功: {}, 角色: {:?}", current_user.username, current_user.role);
                                // 将用户信息存入请求扩展
                                req.extensions_mut().insert(current_user);
                            }
                            Err(e) => {
                                log::warn!("提取用户信息失败: {}", e);
                                // 非公开路径需要返回错误
                                if !is_public {
                                    return Err(ErrorUnauthorized("无效的认证信息"));
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log::warn!("Token验证失败: {}", e);
                        // 非公开路径需要返回错误
                        if !is_public {
                            return Err(ErrorUnauthorized("认证已过期或无效"));
                        }
                    }
                }
            }

            // 对于公开路径，无论认证成功与否都继续处理
            // 对于非公开路径，如果执行到这里说明认证成功
            if is_public {
                log::debug!("公开路径，继续处理: {} {}", method, path);
                return service.call(req).await;
            }

            // 非公开路径，检查是否成功提取了用户信息
            if req.extensions().get::<CurrentUser>().is_none() {
                log::debug!("非公开路径认证失败: {} {}", method, path);
                return Err(ErrorUnauthorized("需要登录"));
            }

            // 继续处理请求
            service.call(req).await
        })
    }
}

impl<S> JwtAuthMiddleware<S> {
    /// 从请求中提取 Token（从 Authorization 头或 Cookie）
    fn extract_token_from_request(req: &ServiceRequest) -> Option<String> {
        // 首先尝试从 Authorization 头中提取
        let auth_header = req
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok());

        if let Some(header) = auth_header {
            if header.starts_with("Bearer ") {
                return Some(header.trim_start_matches("Bearer ").to_string());
            }
        }

        // 然后尝试从 Cookie 中提取
        if let Some(cookie) = req.cookie(ACCESS_TOKEN_COOKIE) {
            return Some(cookie.value().to_string());
        }

        None
    }
}

/// 从请求中提取当前用户（预留接口）
#[allow(dead_code)]
pub fn get_current_user(req: &ServiceRequest) -> Option<CurrentUser> {
    req.extensions().get::<CurrentUser>().cloned()
}

/// 角色检查中间件工厂（预留接口）
#[allow(dead_code)]
pub struct RequireRole {
    role: String,
}

impl RequireRole {
    #[allow(dead_code)]
    pub fn admin() -> Self {
        Self {
            role: "admin".to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn verified() -> Self {
        Self {
            role: "verified".to_string(),
        }
    }
}

/// 需要认证的处理函数包装器（预留接口）
#[allow(dead_code)]
pub async fn auth_required<F, Fut>(
    req: ServiceRequest,
    f: F,
) -> Result<ServiceResponse<actix_web::body::BoxBody>, Error>
where
    F: FnOnce(ServiceRequest) -> Fut,
    Fut: std::future::Future<Output = Result<ServiceResponse<actix_web::body::BoxBody>, Error>>,
{
    if req.extensions().get::<CurrentUser>().is_none() {
        return Err(ErrorUnauthorized("需要登录"));
    }
    f(req).await
}
