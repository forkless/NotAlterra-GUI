// caustic.hlsl - Organic underwater caustic effect
Texture2D inputTexture : register(t0);
SamplerState inputSampler : register(s0);

cbuffer Constants : register(b0)
{
    float time;
    float2 resolution;
    float intensity;
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

float fbm(float2 p)
{
    float sum = 0.0;
    float amp = 0.5;
    float freq = 1.0;
    for (int i = 0; i < 5; i++)
    {
        sum += amp * noise(p * freq);
        amp *= 0.5;
        freq *= 2.0;
    }
    return sum;
}

float4 main(float2 uv : TEXCOORD) : SV_Target
{
    float2 p = uv * 4.0;
    
    // Animated caustic pattern
    float2 q = p + float2(sin(time * 0.3), cos(time * 0.2)) * 0.3;
    float caustic = fbm(q + time * 0.1);
    
    // Multiple layers of caustics
    float caustic2 = fbm(p * 2.0 - time * 0.05);
    float caustic3 = fbm(p * 0.5 + time * 0.08);
    
    // Combine caustics
    float finalCaustic = (caustic * 0.6 + caustic2 * 0.3 + caustic3 * 0.1);
    
    // Color mapping - warm underwater light
    float3 color1 = float3(0.85, 0.95, 1.0);  // Cool blue-white
    float3 color2 = float3(1.0, 0.90, 0.70);  // Warm golden
    float3 color3 = float3(0.60, 0.85, 1.0);  // Deep blue
    
    float3 finalColor = lerp(color1, color2, finalCaustic);
    finalColor = lerp(finalColor, color3, finalCaustic * 0.5);
    
    // Add subtle sparkle
    float sparkle = pow(abs(noise(p * 10.0 + time * 0.5)), 20.0);
    finalColor += sparkle * 0.3;
    
    return float4(finalColor, finalCaustic * intensity * 0.3);
}