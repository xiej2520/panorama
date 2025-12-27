#version 330
out vec4 FragColor;

in vec3 TexCoords;
uniform samplerCube skybox;

uniform float overlayFactor;
uniform sampler2D overlay;
uniform vec2 screenResolution;

void main() {
  // minecraft is slightly brighter with overlay than this implementation, close enough
  vec2 screenUV = gl_FragCoord.xy / screenResolution.xy;
  vec4 overlaySample = texture(overlay, screenUV);
  float alpha = overlaySample.a * overlayFactor;

  FragColor = mix(texture(skybox, TexCoords), overlaySample, alpha);
}
