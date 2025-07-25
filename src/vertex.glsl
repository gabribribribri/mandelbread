#version 130 // SFML often defaults to GLSL 1.30, compatible with most setups

// SFML's automatically provided vertex attributes
in vec2 position;
in vec2 texCoord; // We will use this for the gradient

// SFML's automatically provided transformation uniform
uniform mat4 transform;

// Output to the fragment shader
out vec2 v_texCoord;

void main()
{
    // Apply SFML's transformation matrix to get clip-space position
    gl_Position = transform * vec4(position, 0.0, 1.0);

    // Pass the texture coordinates to the fragment shader
    v_texCoord = texCoord;
}
