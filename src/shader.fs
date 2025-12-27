#version 330
out vec4 FragColor;

in vec3 TexCoords;
uniform samplerCube skybox;
uniform sampler2D overlay;
uniform vec2 screenResolution;

void main() {
  vec2 screenUV = gl_FragCoord.xy / screenResolution.xy;
  vec4 overlaySample = texture(overlay, screenUV);
  float alpha = overlaySample.a;
  
  // minecraft is slightly brighter with overlay, close enough
  FragColor = mix(texture(skybox, TexCoords), overlaySample, alpha);
}
