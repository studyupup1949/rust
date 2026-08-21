// Copyright Jeron A. Lau 2018.
// Dual-licensed under either the MIT License or the Boost Software License,
// Version 1.0.  (See accompanying file LICENSE_1_0.txt or copy at
// https://www.boost.org/LICENSE_1_0.txt)

#version 100
precision mediump float;

attribute vec4 position;
attribute vec4 texpos;
attribute vec4 acolor;

uniform mat4 models_tfm; // The Models' Transform Matrix
uniform mat4 matrix; // The Camera's Transform & Projection Matrix

varying vec4 vcolor;
varying vec4 texcoord;

void main() {
	vec4 place = models_tfm * vec4(position.xyz, 1.0);

	place = matrix * place;

	gl_Position = vec4(place.x, -place.y, place.z, place.w);
	vcolor = acolor;
	texcoord = vec4(texpos.xyz, texpos.w);
}
