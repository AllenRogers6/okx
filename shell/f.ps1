function f {
    param([string]$Pattern)
    if (-not $Pattern) {
        Write-Host "Usage: okx <pattern>"
        return
    }
    if (Test-Path $Pattern -PathType Leaf) {
        okx add $Pattern
        okx open $Pattern
    } else {
        okx open $Pattern
    }
}
