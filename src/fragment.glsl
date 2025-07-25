#version 400 core
#extension GL_ARB_gpu_shader_fp64 : enable
#extension GL_ARB_gpu_shader_int64 : enable

precision highp float;

uniform vec2 u_Resolution;
uniform vec4 u_Center;
uniform vec4 u_Window;
uniform float u_ConvergeDistance;
uniform int u_SeqIter;


vec3 iter_gradient(int iter) {
    float t_norm = float(iter) / float(u_SeqIter);

    vec3 red = vec3(1., 0., 0.);
    vec3 green = vec3(0., 1., 0.);
    vec3 blue = vec3(0., 0., 1.);
    vec3 black = vec3(0., 0., 0.);

    float mid = 0.35;

    float t_seg1 = t_norm / mid;
    vec3 color_seg1 = mix(red, green, t_seg1);

    float t_seg2 = (t_norm - mid) / (1. - mid);
    vec3 color_seg2 = mix(green, blue, t_seg2);

    vec3 final = mix(color_seg1, color_seg2, step(mid, t_norm));
    return final;
}

dvec2 map_pixel_to_value(dvec2 center, dvec2 window, vec2 frag_coord_norm) {
    return dvec2(
        center.x - (window.x/2.) + (frag_coord_norm.x*window.x),
        center.y - (window.y/2.) + (frag_coord_norm.y*window.y)
    );
}

dvec2 sq_add(dvec2 n, dvec2 c) {
    return dvec2(
        n.x * n.x - n.y * n.y + c.x,
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

dvec2 vec4_to_dvec2(vec4 data) {
    
    return dvec2(
        uint64BitsToDouble(uint64_t(floatBitsToUint(data.x)) << 32 | uint64_t(floatBitsToUint(data.y))),
        uint64BitsToDouble(uint64_t(floatBitsToUint(data.z)) << 32 | uint64_t(floatBitsToUint(data.w)))
    );
}

void main()
{
    vec2 frag_coord_norm = gl_FragCoord.xy/u_Resolution;

    dvec2 center = vec4_to_dvec2(u_Center);
    dvec2 window = vec4_to_dvec2(u_Window);

    dvec2 value = map_pixel_to_value(center, window, frag_coord_norm);
    int iter = compute_number_iter(value);

    if (iter == u_SeqIter) {
        gl_FragColor = vec4(vec3(0.), 1.);
    } else {
        vec3 color = iter_gradient(iter);
        gl_FragColor = vec4(color, 1.);
    }
    // double thing = sqrt(value.x*value.x + value.y*value.y);
    // gl_FragColor = vec4(vec3(iter_gradient(int(thing*u_SeqIter))), 1.);
    // gl_FragColor = vec4(vec3(center.y), 1.);
}
