WITH
    bitShiftLeft(1::UInt64, {z:UInt8}) AS zoom_factor,
    bitShiftLeft(1::UInt64, 32 - {z:UInt8}) AS tile_size,

    tile_size * {x:UInt16} AS tile_x_begin,
    tile_size * ({x:UInt16} + 1) AS tile_x_end,

    tile_size * {y:UInt16} AS tile_y_begin,
    tile_size * ({y:UInt16} + 1) AS tile_y_end,

    mercator_x >= tile_x_begin AND mercator_x < tile_x_end
    AND mercator_y >= tile_y_begin AND mercator_y < tile_y_end AS in_tile,

    bitShiftRight(mercator_x - tile_x_begin, 32 - 10 - {z:UInt8}) AS x,
    bitShiftRight(mercator_y - tile_y_begin, 32 - 10 - {z:UInt8}) AS y,

    altitude as pos,

    round(127.5 * (1 + sin(avg(aircraft_wd) * pi() / 180)))::UInt8 AS r,
    round(127.5 * (1 + cos(avg(aircraft_wd) * pi() / 180)))::UInt8 AS g,
    round(127.5 * (1 - sin(avg(aircraft_wd) * pi() / 180)))::UInt8 AS b,
    255::UInt8 AS a  -- Full opacity for the alpha channel

SELECT round(r)::UInt8, round(g)::UInt8, round(b)::UInt8, round(a)::UInt8
FROM {table:Identifier}
WHERE in_tile AND aircraft_wd != 0
GROUP BY pos ORDER BY pos WITH FILL FROM 0 to 1024*1024
