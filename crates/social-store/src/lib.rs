use std::{
    collections::{HashMap, HashSet},
    hash::{DefaultHasher, Hasher},
};

use anyhow::anyhow;
use chrono::Utc;

/// A single Post made by a user that may have interactions (likes and dislikes) as well as comments
/// by other (or the same) users
#[derive(Debug)]
pub struct Post {
    /// The text content of the post
    pub content: String,
    /// A map of whether the post has been liked or disliked by each user that has interacted (at
    /// least liked or disliked) with this post
    likes_and_dislikes: HashMap<String, bool>,
    /// A map of the list of comments left by each user that commented on this post
    comments: HashMap<String, Vec<String>>,
}

impl Post {
    pub fn new<S: Into<String>>(content: S) -> Post {
        Post {
            content: content.into(),
            likes_and_dislikes: HashMap::new(),
            comments: HashMap::new(),
        }
    }

    /// Make the supplied username like this post
    pub fn like(&mut self, author_username: &str) {
        let _ = self
            .likes_and_dislikes
            .insert(author_username.to_string(), true);
    }

    /// Make the supplied username dislike this post
    pub fn dislike(&mut self, author_username: &str) {
        let _ = self
            .likes_and_dislikes
            .insert(author_username.to_string(), false);
    }

    /// Make the supplied username unlike (remove like or dislike) this post
    pub fn unlike(&mut self, author_username: &str) {
        let _ = self.likes_and_dislikes.remove(author_username);
    }

    /// Make the supplied username add the given comment on this post
    pub fn add_comment(&mut self, author_username: &str, content: String) {
        match self.comments.get_mut(author_username) {
            Some(user_comments_this_post) => user_comments_this_post.push(content),
            None => {
                let _ = self
                    .comments
                    .insert(author_username.to_string(), Vec::from([content]));
            }
        }
    }

    /// Return an iterator of the users that have liked this post
    pub fn likers(&self) -> impl Iterator<Item = &String> {
        self.likes_and_dislikes
            .iter()
            .filter_map(|(username, liked_not_disliked)| liked_not_disliked.then_some(username))
    }

    /// Return an iterator of the users that have disliked this post
    pub fn dislikers(&self) -> impl Iterator<Item = &String> {
        self.likes_and_dislikes
            .iter()
            .filter_map(|(username, liked_not_disliked)| (!liked_not_disliked).then_some(username))
    }

    /// Return an iterator of the comments of this post along with the username of the user that
    /// made the comment
    pub fn comments(&self) -> impl Iterator<Item = (&String, &String)> {
        self.comments.iter().flat_map(|(username, comments)| {
            comments.iter().map(move |comment| (username, comment))
        })
    }
}

/// The overall state of the social-media application containing the users and their posts
pub struct State {
    /// A map of the set of the post-ids for the posts made by each registered user on the platform
    users: HashMap<String, HashSet<u64>>,
    /// A map of the indexed collection of posts made by each user on the platform
    posts: HashMap<u64, Post>,
}

impl State {
    pub fn new() -> Self {
        State {
            users: HashMap::new(),
            posts: HashMap::new(),
        }
    }

    /// Register a new user with the supplied username, failing if the username is already
    /// registered
    pub fn register_user(&mut self, username: &str) -> Result<(), anyhow::Error> {
        if self.users.contains_key(username) {
            Err(anyhow!("User `{}` already registered", username))
        } else {
            let _ = self.users.insert(username.to_string(), HashSet::new());
            Ok(())
        }
    }

    /// Make the supplied username create a new post with the supplied text content, returning the
    /// id of the newly created post
    pub fn create_post(&mut self, username: &str, content: String) -> Result<u64, anyhow::Error> {
        use std::hash::Hash;

        let user_posts = self
            .users
            .get_mut(username)
            .ok_or(anyhow!("user `{}` not registered", username))?;

        // insert new post into the state posts
        let new_post = Post::new(content);
        let post_id = {
            let mut hasher = DefaultHasher::new();
            Utc::now().hash(&mut hasher);
            hasher.finish()
        };
        self.posts.insert(post_id, new_post);

        // insert new post's id to user's posts
        user_posts.insert(post_id);
        Ok(post_id)
    }

    /// Make the supplied username create a new comment under the supplied post (identified by its
    /// post-id) with the supplied comment content
    pub fn create_comment(
        &mut self,
        post_id: u64,
        author_username: &str,
        content: String,
    ) -> Result<(), anyhow::Error> {
        let post = self
            .posts
            .get_mut(&post_id)
            .ok_or(anyhow!("post with id `{}` doesn't exist", post_id))?;
        post.add_comment(author_username, content);
        Ok(())
    }

    /// Get an immutable reference to the post identified by the supplied post-id, if it exists
    pub fn get_post(&self, post_id: &u64) -> Option<&Post> {
        self.posts.get(post_id)
    }

    /// Get a mutable reference to the post identified by the supplied post-id, if it exists
    pub fn get_post_mut(&mut self, post_id: &u64) -> Option<&mut Post> {
        self.posts.get_mut(post_id)
    }

    /// Return an iterator over all posts on the platform (identified by their post-ids) along with
    /// the username of the user that posted it
    pub fn posts(&self) -> impl Iterator<Item = (&String, &u64)> {
        self.users.iter().flat_map(|(username, post_ids)| {
            post_ids.iter().map(move |post_id| (username, post_id))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full() -> Result<(), anyhow::Error> {
        let mut state = State::new();

        // no posts have been created
        assert_eq!(state.posts.len(), 0, "");

        // register a user
        state.register_user("bhavyakukkar")?;

        // attempt to register with existing username (must fail)
        assert!(state.register_user("bhavyakukkar").is_err());

        // register another user
        state.register_user("johndoe")?;

        // make bhavyakukkar create a new post
        let post_id = state.create_post("bhavyakukkar", "This is my first post".to_string())?;

        {
            // there is now 1 post
            let posts = state.posts().collect::<Vec<_>>();
            assert!(posts.len() == 1);
            let post_id = *posts.get(0).ok_or(anyhow!("there are no posts"))?.1;
            let post = state
                .get_post_mut(&post_id)
                .ok_or(anyhow!("post not found"))?;

            // post content should match
            assert_eq!(post.content, "This is my first post".to_string());

            // make johndoe like the post
            post.like("johndoe");

            // likers should only be johndoe
            let likers: Vec<&String> = post.likers().collect();
            assert_eq!(likers, vec!["johndoe"]);

            // there should be no dislikers
            let dislikers: Vec<&String> = post.dislikers().collect();
            assert_eq!(dislikers, Vec::<&String>::new());

            // make johndoe disliked the post
            post.dislike("johndoe");

            // there should be no likers
            let likers: Vec<&String> = post.likers().collect();
            assert_eq!(likers, Vec::<&String>::new());

            // dislikers should only be johndoe
            let dislikers: Vec<&String> = post.dislikers().collect();
            assert_eq!(dislikers, vec!["johndoe"]);

            // make johndoe make a comment on the post
            post.add_comment("johndoe", "Nice Post!".to_string());

            // there should be a single comment made by johndoe
            let comments: HashSet<(&String, &String)> = post.comments().collect();
            assert_eq!(
                comments,
                HashSet::from([(&"johndoe".to_string(), &"Nice Post!".to_string())])
            );

            // make johndoe make another comment on the post
            post.add_comment("johndoe", "Reading this again...".to_string());

            // there should now be two comments made by johndoe
            let comments: HashSet<(&String, &String)> = post.comments().collect();
            assert_eq!(
                comments,
                HashSet::from([
                    (&"johndoe".to_string(), &"Nice Post!".to_string()),
                    (&"johndoe".to_string(), &"Reading this again...".to_string())
                ])
            );
        }

        // try creating comment from state rather than post
        // this is bhavyakukkar's comment on his own post
        state.create_comment(
            post_id,
            "bhavyakukkar",
            "Glad to hear that @johndoe".to_string(),
        )?;
        Ok(())
    }
}
