$cert = New-SelfSignedCertificate -Type Custom -Subject "CN=forkless" -FriendlyName "NotAlterra CI" -KeyUsage DigitalSignature -TextExtension @("2.5.29.37={text}1.3.6.1.5.5.7.3.3") -CertStoreLocation "Cert:\CurrentUser\My"
Write-Host "Thumbprint: $($cert.Thumbprint)"
$pwd = ConvertTo-SecureString -String "changeme" -Force -AsPlainText
Export-PfxCertificate -Cert $cert -FilePath "notalterra_cert.pfx" -Password $pwd
Write-Host "Exported to notalterra_cert.pfx"
