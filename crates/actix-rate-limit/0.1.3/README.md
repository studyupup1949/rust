# Rate-Limit middleware for `actix-web`

## 设计方案

本方案参考 [GitHub API v3][github_rate_limiting] 接口，针对用户认证与否，有区别的限流。
比如认证用户，每小时可访问 `600` 次；未认证用户，根据 `ip` 划分，每小时可访问 `60` 次。

## Reference

1. [Everything You Need To Know About API Rate Limiting][api_rate_limiting]
1. [GitHub API v3: Rate limiting][github_rate_limiting]
1. [Redis Pattern: Rate limiter][redis_rate_limiter]

[api_rate_limiting]: https://nordicapis.com/everything-you-need-to-know-about-api-rate-limiting/
[github_rate_limiting]: https://developer.github.com/v3/#rate-limiting
[redis_rate_limiter]: https://redis.io/commands/incr#pattern-rate-limiter

## License

[Apache](LICENSE-APACHE) or [MIT](LICENSE-MIT)
