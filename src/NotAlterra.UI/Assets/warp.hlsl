// warp.hlsl - Fluid distortion effect
Texture2D inputTexture : register(t0);
SamplerState inputSampler : register(s0);

cbuffer Constants : register(b0)
{
    float time;
    float2 resolution;
    float warpAmount;
}

float2 hash(float2 p)
{
    p = float2(dot(p, float2(127.1, 311.7)),
               dot(p, float2(269.5, 183.3)));
    return -1.0 + 2.0 * frac(sin(p) * 43758.5453123);
}

float noise(float2 p)
{
    float2 i = floor(p);
    float2 f = frac(p);
    float2 u = f * f * (3.0 - 2.0 * f);
    return lerp(lerp(dot(hash(i + float2(0.0, 0.0)), f - float2(0.0, 0.0)),
                     dot(hash(i + float2(1.0, 0.0)), f - float2(1.0, 0.0)), u.x),
                lerp(dot(hash(i + float2(0.0, 1.0)), f - float2(0.0, 1.0)),
                     dot(hash(i + float2(1.0, 1.0)), f - float2(1.0, 1.0)), u.x), u.y);
}

float4 main(float2 uv : TEXCOORD) : SV_Target
{
    // Generate distortion map
    float timeOffset = time * 0.2;
    float2 distortion = float2(
        noise(float2(uv.x * 3.0 + timeOffset, uv.y * 3.0 + 1.7)),
        noise(float2(uv.x * 3.0 + 0.3, uv.y * 3.0 + timeOffset + 1.2))
    );
    
    // Apply warp
    float2 warpedUV = uv + (distortion - 0.5) * warpAmount * 0.05;
    warpedUV = frac(warpedUV); // Wrap around edges
    
    // Sample texture with warped UV
    float4 color = inputTexture.Sample(inputSampler, warpedUV);
    
    // Add subtle vignette
    float2 vignette = uv - 0.5;
    float v = 1.0 - dot(vignette, vignette) * 0.3;
    color.rgb *= v;
    
    return color;
}