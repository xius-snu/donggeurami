# Makes the app's icons, for iOS and Android, from appiconroundtown.png
# beside this script: Hajun's picture of the town (since 2026-10-08).
#
#   powershell -ExecutionPolicy Bypass -File mobile\make_icons.ps1
#
# Run it again after changing the picture. What it writes is committed, so
# neither Xcode, Gradle nor CI runs this.
#
# iOS: ios\Assets.xcassets\AppIcon.appiconset\icon-1024.png. Two App Store
# rules drive it: exactly 1024x1024, and no alpha channel (an icon with
# transparency is rejected at upload), hence Format24bppRgb. iOS rounds the
# corners itself.
#
# Android (minSdk 31, so adaptive icons only): ic_launcher_foreground.png in
# res\mipmap-<density>, 108 dp square, which res\mipmap-anydpi\ic_launcher.xml
# names. The launcher masks it to its own shape, a squircle on Samsung and a
# circle on a Pixel, and shows only the middle 72 dp of it. The whole picture
# is fitted to those 72 dp, so that it shows whole as on iOS; the 18 dp round
# it, seen only when a launcher moves the icon about, is the picture's own
# edge drawn out, more sky above and sea below.

Add-Type -AssemblyName System.Drawing

$picture = [System.Drawing.Image]::FromFile((Join-Path $PSScriptRoot 'appiconroundtown.png'))

function New-Square([int]$size) {
    $bmp = New-Object System.Drawing.Bitmap($size, $size, [System.Drawing.Imaging.PixelFormat]::Format24bppRgb)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.CompositingQuality = [System.Drawing.Drawing2D.CompositingQuality]::HighQuality
    $g.PixelOffsetMode    = [System.Drawing.Drawing2D.PixelOffsetMode]::Half
    return $bmp, $g
}

# Draws the picture $size pixels square, inside a margin $inset pixels wide
# that its edges are drawn out across, and saves it to $out.
function Save-Icon([int]$size, [int]$inset, [string]$out) {
    # The picture scaled, its edges mirrored for the filter to read past them:
    # without that, GDI+'s bicubic filter blends the border with nothing and
    # leaves a faint dark line round it.
    $inner = $size - 2 * $inset
    $scaled, $g = New-Square $inner
    $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $mirrored = New-Object System.Drawing.Imaging.ImageAttributes
    $mirrored.SetWrapMode([System.Drawing.Drawing2D.WrapMode]::TileFlipXY)
    $g.DrawImage($picture, (New-Object System.Drawing.Rectangle(0, 0, $inner, $inner)),
        0, 0, $picture.Width, $picture.Height, [System.Drawing.GraphicsUnit]::Pixel, $mirrored)
    $g.Dispose(); $mirrored.Dispose()

    # Then in the middle of the icon, with its outermost row of pixels on each
    # side stretched across the margin, and its corner pixels into the corners.
    $bmp, $g = New-Square $size
    $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::NearestNeighbor
    $far = $inset + $inner
    $last = $inner - 1
    $pieces = @(
        # to x, y, width, height       from x, y, width, height
        @($inset, $inset, $inner, $inner,   0, 0, $inner, $inner),
        @(0, $inset, $inset, $inner,         0, 0, 1, $inner),
        @($far, $inset, $inset, $inner,      $last, 0, 1, $inner),
        @($inset, 0, $inner, $inset,         0, 0, $inner, 1),
        @($inset, $far, $inner, $inset,      0, $last, $inner, 1),
        @(0, 0, $inset, $inset,              0, 0, 1, 1),
        @($far, 0, $inset, $inset,           $last, 0, 1, 1),
        @(0, $far, $inset, $inset,           0, $last, 1, 1),
        @($far, $far, $inset, $inset,        $last, $last, 1, 1)
    )
    foreach ($p in $pieces) {
        if ($p[2] -le 0 -or $p[3] -le 0) { continue }
        $to = New-Object System.Drawing.Rectangle($p[0], $p[1], $p[2], $p[3])
        $from = New-Object System.Drawing.Rectangle($p[4], $p[5], $p[6], $p[7])
        $g.DrawImage($scaled, $to, $from, [System.Drawing.GraphicsUnit]::Pixel)
    }

    New-Item -ItemType Directory -Force (Split-Path $out) | Out-Null
    $bmp.Save($out, [System.Drawing.Imaging.ImageFormat]::Png)
    $g.Dispose(); $bmp.Dispose(); $scaled.Dispose()
    Write-Output "wrote $out"
}

Save-Icon 1024 0 (Join-Path $PSScriptRoot 'ios\Assets.xcassets\AppIcon.appiconset\icon-1024.png')

# 108 dp at each density, an 18 dp margin round the picture.
$res = Join-Path $PSScriptRoot 'android\app\src\main\res'
foreach ($density in @(@('mdpi', 1.0), @('hdpi', 1.5), @('xhdpi', 2.0), @('xxhdpi', 3.0), @('xxxhdpi', 4.0))) {
    $scale = $density[1]
    Save-Icon ([int](108 * $scale)) ([int](18 * $scale)) (Join-Path $res "mipmap-$($density[0])\ic_launcher_foreground.png")
}

$picture.Dispose()
