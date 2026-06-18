const STATUS_CLASS_DIVISOR: u16 = 100;
pub const NOT_FOUND: u16 = 404;
const SUCCESS_MIN: u16 = 200;
const SUCCESS_MAX_EXCLUSIVE: u16 = 300;
const REDIRECT_MAX_EXCLUSIVE: u16 = 400;
const UNAUTHORIZED: u16 = 401;
const FORBIDDEN: u16 = 403;

pub fn status_class(status: u16) -> u16 {
    status / STATUS_CLASS_DIVISOR
}

pub fn is_success(status: u16) -> bool {
    (SUCCESS_MIN..SUCCESS_MAX_EXCLUSIVE).contains(&status)
}

/// Operation reached its handler: 2xx, or a 3xx we don't follow (POST-redirect-GET).
pub fn reached_operation(status: u16) -> bool {
    (SUCCESS_MIN..REDIRECT_MAX_EXCLUSIVE).contains(&status)
}

/// The server actively refused the actor (the access-control signal we need a differential on).
pub fn is_denied(status: u16) -> bool {
    matches!(status, UNAUTHORIZED | FORBIDDEN)
}
