#version 330

// Input from the vertex shader (interpolated)
in vec2 v_texCoord;

// Output color for the fragment
out vec4 FragColor;

void main()
{
    // Calculate the red and green components based on the x-coordinate
    float red = 1.0 - v_texCoord.x; // Red goes from 1.0 to 0.0
    float green = v_texCoord.x;     // Green goes from 0.0 to 1.0
    float blue = 0.0;               // No blue component

    // Set the final color
    FragColor = vec4(red, green, blue, 1.0); // Full opacity
}
