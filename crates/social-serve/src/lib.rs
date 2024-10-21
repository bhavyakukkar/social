use std::fmt::Write;

use social_store::{Post, State};

/// Just a HTML header
const HEADER: &str = "<!DOCTYPE html>
    <html lang=\"ko\">
    <head>
    	<meta http-equiv=\"Content-Type\" content=\"text/html; charset=utf-8\" />
    	<meta name=\"description\" content=\"\" />
    	<meta name=\"author\" content=\"\" />
    	<meta name=\"viewport\" content=\"user-scalable=no, initial-scale=1.0, maximum-scale=1.0, minimum-scale=1.0, width=device-width\" />
    	<title></title>
    	<link href=\"css/style.css\" rel=\"stylesheet\" />
    </head>
    <body>
        <h1>Social</h1>";

/// Just a HTML footer
const FOOTER: &str = "</body></html>";

/// Trait implemented for types that can be rendered to HTML
pub trait ToHtml {
    /// Trait method that writes the HTML string content to a provided writable string
    fn to_html(&self, s: &mut String) -> std::fmt::Result;
}

/// A post-view that access a post as well as its author
pub struct PostView<'a> {
    pub post_id: &'a u64,
    pub author: &'a String,
    pub post: &'a Post,
}

impl<'a> ToHtml for PostView<'a> {
    fn to_html(&self, s: &mut String) -> std::fmt::Result {
        let likers = self
            .post
            .likers()
            .fold(String::new(), |s, liker| s + " \"" + liker + "\"");

        let dislikers = self
            .post
            .dislikers()
            .fold(String::new(), |s, disliker| s + " \"" + disliker + "\"");

        let comments = self
            .post
            .comments()
            .fold(String::new(), |s, (username, comment)| {
                s + &format!("<li><b>@{username} says:</b> {comment}</li>")
            });

        s.write_str(HEADER)?;
        write!(
            s,
            "
                <h2>Post by @{author}</h2>
                <h4>{content}</h4>
                <p>Liked by {likers}</p>
                <p>Disliked by {dislikers}</p>
                <h4>Comments</h4>
                <ul>
                    {comments}
                    <li>
                        <form action=\"/add-comment\" method=\"GET\">
                            <input hidden name=\"post_username\" value=\"{author}\"/>
                            <input hidden name=\"post_id\" value=\"{post_id}\"/>
                            <input name=\"username\" placeholder=\"Username\"/>
                            <input name=\"comment\" placeholder=\"Your Comment\"/>
                            <input type=\"submit\" value=\"Add Comment\"/>
                        </form>
                    </li>
                </ul>

                <form action=\"/like\" method=\"GET\">
                    <input hidden name=\"post_username\" value=\"{author}\"/>
                    <input hidden name=\"post_id\" value=\"{post_id}\"/>
                    <input name=\"username\" placeholder=\"Username\"/>
                    <input type=\"submit\" value=\"Like\"/>
                </form>
                <form action=\"/dislike\" method=\"GET\">
                    <input hidden name=\"post_username\" value=\"{author}\"/>
                    <input hidden name=\"post_id\" value=\"{post_id}\"/>
                    <input name=\"username\" placeholder=\"Username\"/>
                    <input type=\"submit\" value=\"Dislike\"/>
                </form>
                <form action=\"/unlike\" method=\"GET\">
                    <input hidden name=\"post_username\" value=\"{author}\"/>
                    <input hidden name=\"post_id\" value=\"{post_id}\"/>
                    <input name=\"username\" placeholder=\"Username\"/>
                    <input type=\"submit\" value=\"Unlike\"/>
                </form>
                <h5><a href=\"/feed\">Back to Feed</h5>
            ",
            post_id = self.post_id,
            author = self.author,
            content = self.post.content,
        )?;
        s.write_str(FOOTER)
    }
}

impl ToHtml for State {
    fn to_html(&self, s: &mut String) -> std::fmt::Result {
        let posts = self
            .posts()
            .filter_map(|(username, post_id)| {
                self.get_post(post_id).map(|post| (username, post, post_id))
            })
            .fold(
                String::from("<div id=\"posts\" style=\"border: solid 1px black;\">"),
                |mut s, (username, post, post_id)| {
                    let _ = write!(s, "<a href=\"/post/{}/{}\"><div>", username, post_id);
                    let post = PostView {
                        post_id,
                        author: username,
                        post,
                    };
                    let _ = post.to_html(&mut s);
                    let _ = s.write_str("</div></a>");
                    s
                },
            )
            + "</div>";
        s.write_str(HEADER)?;
        s.write_str(&posts)?;
        s.write_str(FOOTER)
    }
}
