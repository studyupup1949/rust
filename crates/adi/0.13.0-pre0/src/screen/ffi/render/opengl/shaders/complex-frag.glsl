// Copyright Jeron A. Lau 2018.
// Dual-licensed under either the MIT License or the Boost Software License,
// Version 1.0.  (See accompanying file LICENSE_1_0.txt or copy at
// https://www.boost.org/LICENSE_1_0.txt)

#version 100
precision mediump float;

uniform sampler2D texture;

varying vec4 texcoord;
varying vec4 vcolor;

void main() {
	vec4 sampled = texture2D(texture, texcoord.xy);
	vec4 out_color = vec4(sampled.rgb, sampled.a * texcoord.a) * vcolor;

	gl_FragColor = out_color;
}
