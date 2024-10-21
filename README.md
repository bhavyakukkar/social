+ the project is split into 2 crates: `social-store` that manages the storing of users and posts,
  and `social-serve` that manages the rendering of posts and comments and starts an axum server

+ Build & serve at `localhost:8000`:
```sh
cargo run
```

+ First, register a user by going to [http://localhost:8000/register/johndoe](http://localhost:8000/register/johndoe)
+ Next, create a post by going to [http://localhost:8000/new-post/johndoe/first%20post](http://localhost:8000/new-post/johndoe/first%20post)
+ After that, you can start navigating the website by going to [http://localhost:8000/feed](http://localhost:8000/feed)

+ Known limitation: comments can be made without having to register

+ run tests:
```sh
cargo test -p social-store
```
