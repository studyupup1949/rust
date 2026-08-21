-- Rate limit by each hour
-- * KEYS[1]: id
-- * KEYS[2]: hours
-- * ARGV[1]: limit

local limit = tonumber(ARGV[1])
local key = KEYS[1] .. ":" .. KEYS[2]
local hit = redis.call("INCR", key)
if hit == 1 then
    redis.call("EXPIRE", key, 3600)
end

return math.max(0, limit - hit)
