use std::{rc::Rc, sync::Arc, task::{Context, Poll}};
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::header,
    Error, HttpMessage,
};
use actix_web::cookie::Cookie;
use futures::future::{ready, Ready, LocalBoxFuture};
use uuid::Uuid;

use crate::{
    db::db::UserExt,
    errors::error::{ErrorMessage, HttpError},
    models::models::{User, UserRole},
    utils::token,
    AppState,
};