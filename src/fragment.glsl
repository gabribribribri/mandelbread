#version 400 core
#extension GL_ARB_gpu_shader_fp64 : enable
#extension GL_ARB_gpu_shader_int64 : enable

precision highp float;

uniform vec2 u_Resolution;
uniform ivec4 u_Center;
uniform ivec4 u_Window;
uniform float u_ConvergeDistance;
uniform int u_SeqIter;


vec3 iter_gradient(int iter) {
    float t_norm = iter / u_SeqIter;

    vec3 red = vec3(1., 0., 0.);
    vec3 green = vec3(0., 1., 0.);
    vec3 blue = vec3(0., 0., 1.);
    vec3 black = vec3(0., 0., 0.);

    float mid = 0.35;

    float t_seg1 = t_norm / mid;
    vec3 color_seg1 = mix(blue, green, t_seg1);

    float t_seg2 = (t_norm - mid) / (1. - mid);
    vec3 color_seg2 = mix(green, red, t_seg2);

    vec3 final = mix(color_seg1, color_seg2, step(mid, t_norm));
    vec3 final_or_black = mix(final, black, step(1., t_norm));
    return final_or_black;
}

dvec2 map_pixel_to_value(dvec2 center, dvec2 window, vec2 frag_coord_norm) {
    return dvec2(
        center.x - (window.x/2.) + (frag_coord_norm.x*window.x),
        center.y - (window.y/2.) + (frag_coord_norm.y*window.y)
    );
}

dvec2 sq_add(dvec2 n, dvec2 c) {
    return dvec2(
        n.x * n.x + n.y * n.y + c.x,
        2. * n.x * n.y + c.y
    );
}

int compute_number_iter(dvec2 c) {
    double distance = 0.;
    dvec2 n = c;
    int iter = 0;
    for (; iter < u_SeqIter && distance <= double(u_ConvergeDistance); iter++) {
        n = sq_add(n, c);
        distance = abs(n.x) + abs(n.y);
    }
    return iter;
}


void main()
{
    vec2 frag_coord_norm = vec2(gl_FragCoord.x/u_Resolution.x, gl_FragCoord.y/u_Resolution.y);

    dvec2 center = dvec2(
        int64BitsToDouble(int64_t(u_Center.x) << 32 | int64_t(u_Center.y)),
        int64BitsToDouble(int64_t(u_Center.z) << 32 | int64_t(u_Center.w))
    );
    dvec2 window = dvec2(
        int64BitsToDouble(int64_t(u_Window.x) << 32 | int64_t(u_Window.y)),
        int64BitsToDouble(int64_t(u_Window.z) << 32 | int64_t(u_Window.w))
    );

    dvec2 value = map_pixel_to_value(center, window, frag_coord_norm);
    int iter = compute_number_iter(value);
    vec3 color = iter_gradient(iter);

    gl_FragColor = vec4(color, 1.);
    // gl_FragColor = vec4(vec3(abs(value.x) + abs(value.y)), 1.);
}
