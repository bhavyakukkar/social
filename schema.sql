CREATE TABLE user (
    username VARCHAR(255) NOT NULL PRIMARY KEY
);

CREATE TABLE post (
    post_id BIGINT NOT NULL PRIMARY KEY,
    content VARCHAR(1023) NOT NULL,
    username VARCHAR(255) NOT NULL,

    FOREIGN KEY (username) REFERENCES user(username)
);

CREATE TABLE interaction (
    like_not_dislike BOOLEAN NOT NULL,
    post_id BIGINT NOT NULL, 
    username VARCHAR(255) NOT NULL,

    FOREIGN KEY (post_id) REFERENCES post(post_id),
    FOREIGN KEY (username) REFERENCES user(username)
);

CREATE TABLE comment (
    content VARCHAR(1023) NOT NULL,
    post_id BIGINT NOT NULL, 
    username VARCHAR(255) NOT NULL,

    FOREIGN KEY (post_id) REFERENCES post(post_id),
    FOREIGN KEY (username) REFERENCES user(username)
);
