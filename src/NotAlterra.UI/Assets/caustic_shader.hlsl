float time : register(c0);
float2 resolution : register(c1);
float intensity : register(c2);

float4 main(float4 pos : SV_Position, float2 uv : TEXCOORD) : SV_Target
{
    // TEST: solid red with alpha based on intensity
    return float4(1.0, 0, 0, intensity * 0.5);
}
