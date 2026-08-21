// Copyright Jeron A. Lau 2018.
// Dual-licensed under either the MIT License or the Boost Software License,
// Version 1.0.  (See accompanying file LICENSE_1_0.txt or copy at
// https://www.boost.org/LICENSE_1_0.txt)

#version 100
precision mediump float;

uniform sampler2D texture;

varying vec4 texcoord;

void main() {
    gl_FragColor = texture2D(texture, texcoord.xy);
}
