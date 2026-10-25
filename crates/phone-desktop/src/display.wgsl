@group(0) @binding(0) var framebuffer: texture_2d<f32>;
@group(0) @binding(1) var nearest: sampler;
struct Vertex { @builtin(position) position: vec4<f32>, @location(0) uv: vec2<f32> }
@vertex fn vs(@builtin(vertex_index) index:u32) -> Vertex {
 let points=array<vec2<f32>,3>(vec2(-1.,-1.),vec2(3.,-1.),vec2(-1.,3.));
 var out:Vertex;let p=points[index];out.position=vec4(p,0.,1.);out.uv=vec2((p.x+1.)/2.,(1.-p.y)/2.);return out;
}
@fragment fn fs(in:Vertex) -> @location(0) vec4<f32> {return textureSample(framebuffer,nearest,in.uv);}
