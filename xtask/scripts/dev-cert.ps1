# User-level test-certificate steps for `cargo xtask dev-cert` and `dev-install`.
param(
    [Parameter(Mandatory)][ValidateSet('Find', 'List', 'Create', 'Export', 'Remove')][string]$Action,
    [Parameter(Mandatory)][string]$Subject,
    [Parameter(Mandatory)][string]$Store,
    [int]$Days,
    [int]$MinDays,
    [string]$Digest,
    [string]$Thumbprint,
    [string]$CertFile
)
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [Text.Encoding]::UTF8
# The current user's store; the private key is created there and never exported.
$Path = "Cert:\CurrentUser\$Store"
# Marks the certificate as an end entity, so it can never vouch for other certificates.
$NotCa = '2.5.29.19={critical}{text}ca=false'

# Every certificate in the store with our subject.
function Get-Ours {
    Get-ChildItem -LiteralPath $Path | Where-Object { $_.Subject -eq $Subject }
}

switch ($Action) {
    'Find' {
        $until = (Get-Date).AddDays($MinDays)
        Get-Ours | Where-Object { $_.HasPrivateKey -and $_.NotAfter -gt $until } |
            Sort-Object NotAfter -Descending |
            Select-Object -First 1 -ExpandProperty Thumbprint
    }
    'List' { Get-Ours | Select-Object -ExpandProperty Thumbprint }
    'Create' {
        $cert = New-SelfSignedCertificate -Type CodeSigningCert -Subject $Subject `
            -CertStoreLocation $Path -KeyExportPolicy NonExportable -KeyUsage DigitalSignature `
            -TextExtension @($NotCa) -HashAlgorithm $Digest -NotAfter (Get-Date).AddDays($Days)
        $cert.Thumbprint
    }
    'Export' {
        $cert = Get-Item -LiteralPath (Join-Path $Path $Thumbprint)
        Export-Certificate -Cert $cert -FilePath $CertFile -Type CERT | Out-Null
    }
    'Remove' { Get-Ours | ForEach-Object { Remove-Item -LiteralPath $_.PSPath -DeleteKey } }
}
