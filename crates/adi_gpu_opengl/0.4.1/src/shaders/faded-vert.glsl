// "adi_gpu_opengl" crate - Licensed under the MIT LICENSE
//  * Copyright (c) 2018  Jeron A. Lau <jeron.lau@plopgrizzly.com>

#version 100
precision mediump float;

attribute vec4 position;
attribute vec4 texpos;

uniform mat4 models_tfm; // The Models' Transform Matrix
uniform int has_camera; // 0 no, 1 yes, 2 fog
uniform mat4 matrix; // The Camera's Transform & Projection Matrix

uniform float alpha; // This shader's uniform.

varying vec4 texcoord;
varying float z;

void main() {
	vec4 place = models_tfm * vec4(position.xyz, 1.0);

	if(has_camera == 1) {
		place = matrix * place;
	}

	gl_Position = vec4(place.x, -place.y, place.z, place.w);
	texcoord = vec4(texpos.xyz, texpos.w * alpha);
	z = length(place.xyz);
}
