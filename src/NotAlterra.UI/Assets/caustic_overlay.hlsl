Texture2D<float4> InputTexture : register(t0);
SamplerState InputSampler : register(s0);

float time : register(c0);
float2 resolution : register(c1);
float strength : register(c2);

float4 main(float4 pos : SV_Position, float2 uv : TEXCOORD) : SV_Target
{
    // ── UV displacement (shimmer) ──
    float2 dUV = uv;
    dUV.x += sin(time + uv.y * 15.0) * 0.01 * strength;
    dUV.y += cos(time * 0.8 + uv.x * 12.0) * 0.01 * strength;

    float4 col = InputTexture.Sample(InputSampler, dUV);

    // ── Blue color grading ──
    col.r *= 0.85;
    col.g *= 0.92;
    col.b *= 1.15;

    // ── Procedural caustics ──
    float c1 = sin(uv.x * 30.0 + time * 1.0) * cos(uv.y * 25.0 + time * 0.7);
    float c2 = sin(uv.y * 35.0 + time * 0.5) * cos(uv.x * 20.0 + time * 0.9);
    float caustic = max(0, c1) * max(0, c2);
    caustic = pow(caustic, 1.5) * 0.25 * strength;
    col.rgb += float3(0.15, 0.35, 0.55) * caustic;

    // ── Vignette ──
    float2 ctr = uv - 0.5;
    float vig = 1.0 - dot(ctr, ctr) * 1.2;
    vig = clamp(vig, 0, 1);
    col.rgb *= lerp(0.3, 1.0, vig);

    // ── Fake bloom ──
    float lum = dot(col.rgb, float3(0.299, 0.587, 0.114));
    float bloom = max(0, lum - 0.5) * 0.4;
    col.rgb += float3(bloom * 0.7, bloom * 0.85, bloom * 1.3);

    return saturate(col);
}
