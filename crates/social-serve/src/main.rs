use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::{Path as AxumPath, Query as AxumQuery, State as AxumState},
    response::{Html, Redirect},
    routing::get,
    Router,
};
use serde::Deserialize;
use social_serve::{PostView, ToHtml};
use social_store::State;
use tokio::{net::TcpListener, sync::RwLock};

type SharedState = Arc<RwLock<State>>;

async fn all_posts(AxumState(state): AxumState<SharedState>) -> Result<Html<String>, String> {
    let mut respone = String::new();
    state
        .read_owned()
        .await
        .to_html(&mut respone)
        .map_err(|err| err.to_string())?;
    Ok(Html(respone))
}

async fn one_post(
    AxumState(state): AxumState<SharedState>,
    AxumPath((username, post_id)): AxumPath<(String, u64)>,
) -> Result<Html<String>, String> {
    let mut response = String::new();
    let read_state = state.read_owned().await;
    let post_view = PostView {
        post_id: &post_id,
        author: &username,
        post: read_state
            .get_post(&post_id)
            .ok_or("Post not found".to_string())?,
    };
    let _ = post_view.to_html(&mut response);
    Ok(Html(response))
}

async fn create_post(
    AxumState(state): AxumState<SharedState>,
    AxumPath((username, content)): AxumPath<(String, String)>,
) -> Result<Redirect, String> {
    let mut write_state = state.write_owned().await;
    let new_post_id = write_state
        .create_post(&username, content)
        .map_err(|err| err.to_string())?;
    Ok(Redirect::permanent(&format!(
        "/post/{username}/{new_post_id}"
    )))
}

async fn register_user(
    AxumState(state): AxumState<SharedState>,
    AxumPath(username): AxumPath<String>,
) -> Result<Redirect, String> {
    let mut write_state = state.write_owned().await;
    write_state
        .register_user(&username)
        .map_err(|err| err.to_string())?;
    Ok(Redirect::permanent("/feed"))
}

#[derive(Deserialize)]
struct CreateComment {
    post_id: u64,
    post_username: String,
    username: String,
    comment: String,
}
async fn create_comment(
    AxumState(state): AxumState<SharedState>,
    AxumQuery(CreateComment {
        post_id,
        post_username,
        username,
        comment,
    }): AxumQuery<CreateComment>,
) -> Result<Redirect, String> {
    state
        .write_owned()
        .await
        .create_comment(post_id, &username, comment)
        .map_err(|err| err.to_string())?;
    Ok(Redirect::permanent(&format!(
        "/post/{post_username}/{post_id}"
    )))
}

#[derive(Deserialize)]
struct Like {
    post_id: u64,
    post_username: String,
    username: String,
}
async fn like(
    AxumState(state): AxumState<SharedState>,
    AxumQuery(Like {
        post_id,
        username,
        post_username,
    }): AxumQuery<Like>,
) -> Result<Redirect, String> {
    let mut write_state = state.write_owned().await;
    let post = write_state
        .get_post_mut(&post_id)
        .ok_or("Post not found".to_string())?;
    post.like(&username);
    Ok(Redirect::permanent(&format!(
        "/post/{post_username}/{post_id}"
    )))
}

async fn dislike(
    AxumState(state): AxumState<SharedState>,
    AxumQuery(Like {
        post_id,
        username,
        post_username,
    }): AxumQuery<Like>,
) -> Result<Redirect, String> {
    let mut write_state = state.write_owned().await;
    let post = write_state
        .get_post_mut(&post_id)
        .ok_or("Post not found".to_string())?;
    post.dislike(&username);
    Ok(Redirect::permanent(&format!(
        "/post/{post_username}/{post_id}"
    )))
}

async fn unlike(
    AxumState(state): AxumState<SharedState>,
    AxumQuery(Like {
        post_id,
        username,
        post_username,
    }): AxumQuery<Like>,
) -> Result<Redirect, String> {
    let mut write_state = state.write_owned().await;
    let post = write_state
        .get_post_mut(&post_id)
        .ok_or("Post not found".to_string())?;
    post.unlike(&username);
    Ok(Redirect::permanent(&format!(
        "/post/{post_username}/{post_id}"
    )))
}

#[tokio::main]
async fn main() {
    let app_state = State::new();

    let router = Router::new()
        .route("/feed", get(all_posts))
        .route("/post/:username/:id", get(one_post))
        .route("/register/:username", get(register_user))
        .route("/new-post/:username/:content", get(create_post))
        .route("/add-comment", get(create_comment))
        .route("/like", get(like))
        .route("/dislike", get(dislike))
        .route("/unlike", get(unlike))
        .route("/", get(|| async { Redirect::permanent("/feed") }));

    let app = router.with_state(Arc::new(RwLock::new(app_state)));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    let listener = TcpListener::bind(&addr).await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
