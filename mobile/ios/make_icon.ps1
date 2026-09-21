# Generates mobile/ios/Assets.xcassets/AppIcon.appiconset/icon-1024.png
#
#   powershell -ExecutionPolicy Bypass -File mobile\ios\make_icon.ps1
#
# Run it again after editing the colours below; the PNG is committed, so CI
# never runs this.
#
# Two App Store rules drive the output format: the icon must be exactly
# 1024x1024, and it must have no alpha channel (an icon with transparency is
# rejected at upload). Hence Format24bppRgb.
#
# The palette is the game's own — SKY_DAY and the cloud/land colours from
# src/day_night.rs and src/lib.rs — so the icon and the first frame agree.
#
# There is deliberately no text. At the 60x60 the icon is actually drawn at on
# a home screen, "동그라미타운" is an illegible smudge, and Apple's Human
# Interface Guidelines advise against wordmarks in app icons. The name already
# sits under the icon, from CFBundleDisplayName.

Add-Type -AssemblyName System.Drawing

$size = 1024
$bmp  = New-Object System.Drawing.Bitmap($size, $size, [System.Drawing.Imaging.PixelFormat]::Format24bppRgb)
$g    = [System.Drawing.Graphics]::FromImage($bmp)
$g.SmoothingMode     = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
$g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic

function C([int]$r, [int]$gr, [int]$b) { [System.Drawing.Color]::FromArgb(255, $r, $gr, $b) }

$skyTop   = C 106 186 224   # SKY_DAY, a shade deeper at the zenith
$skyLow   = C 190 228 242   # paler toward the horizon
$ocean    = C  43 108 163
$sand     = C 232 213 163
$land     = C  92 140  87   # Color::srgb(0.36, 0.55, 0.34)
$tree     = C  63 107  60
$sun      = C 255 209  51   # the HUD's jump-button gold

$horizon = 620

# Sky
$skyRect  = New-Object System.Drawing.Rectangle(0, 0, $size, $horizon)
$skyBrush = New-Object System.Drawing.Drawing2D.LinearGradientBrush($skyRect, $skyTop, $skyLow, 90.0)
$g.FillRectangle($skyBrush, $skyRect)

# Ocean
$g.FillRectangle((New-Object System.Drawing.SolidBrush($ocean)), 0, $horizon, $size, $size - $horizon)

# Sun, kept clear of the corner that iOS rounds off
$g.FillEllipse((New-Object System.Drawing.SolidBrush($sun)), 740 - 110, 300 - 110, 220, 220)

# Island: a sand rim with the land circle on top
$cx = 512; $cy = 668; $r = 236
$g.FillEllipse((New-Object System.Drawing.SolidBrush($sand)), $cx - $r - 26, $cy - $r - 26, ($r + 26) * 2, ($r + 26) * 2)
$g.FillEllipse((New-Object System.Drawing.SolidBrush($land)), $cx - $r, $cy - $r, $r * 2, $r * 2)

# A few round trees, because the town is made of circles
$treeBrush = New-Object System.Drawing.SolidBrush($tree)
foreach ($t in @(@(432, 556, 44), @(556, 520, 56), @(646, 582, 38))) {
    $g.FillEllipse($treeBrush, $t[0] - $t[2], $t[1] - $t[2], $t[2] * 2, $t[2] * 2)
}

$out = Join-Path $PSScriptRoot 'Assets.xcassets\AppIcon.appiconset\icon-1024.png'
$bmp.Save($out, [System.Drawing.Imaging.ImageFormat]::Png)
$g.Dispose(); $bmp.Dispose()
Write-Output "wrote $out"
