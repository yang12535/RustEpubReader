package com.epub.reader.ui.pagecurl

import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawWithCache
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.drawscope.clipRect
import androidx.compose.ui.graphics.drawscope.withTransform
import androidx.compose.ui.unit.dp

/**
 * Front-page layer: clips content to [spineX, foldX] (still-flat part).
 * No shadow here; shadow is a separate top-level layer.
 */
internal fun Modifier.drawBookSpreadFront(
    foldFraction: Float,
): Modifier = drawWithCache {
    val w = size.width
    val h = size.height
    val spineX = w / 2f
    val halfW = w / 2f
    val foldX = spineX + halfW * foldFraction

    if (foldFraction >= 0.999f) {
        return@drawWithCache onDrawWithContent {
            clipRect(spineX, 0f, w, h) {
                this@onDrawWithContent.drawContent()
            }
        }
    }
    if (foldFraction <= 0.001f) {
        return@drawWithCache onDrawWithContent { /* fully turned */ }
    }

    onDrawWithContent {
        clipRect(spineX, 0f, foldX, h) {
            this@onDrawWithContent.drawContent()
        }
    }
}

/**
 * Back-flap layer: the folded-over page on TOP of the front page.
 *
 * Shows next spread's left page content, translated so that
 * content x=spineX aligns with screen x=foldX. Region: [backLeft, foldX].
 *
 * Includes a subtle depth gradient (darker near leading edge).
 */
internal fun Modifier.drawBookSpreadBackFlap(
    foldFraction: Float,
    depthAlpha: Float = 0.10f,
): Modifier = drawWithCache {
    val w = size.width
    val h = size.height
    val spineX = w / 2f
    val foldX = spineX + spineX * foldFraction
    val backLeft = (2f * foldX - w).coerceAtLeast(0f)

    if (foldFraction >= 0.999f || backLeft >= foldX) {
        return@drawWithCache onDrawWithContent { /* nothing */ }
    }

    val tx = 2f * foldX - w

    onDrawWithContent {
        clipRect(backLeft, 0f, foldX, h) {
            withTransform({
                translate(left = tx, top = 0f)
            }) {
                this@onDrawWithContent.drawContent()
            }

            // Optional subtle shading at the fold crease (darker at foldX, fading towards backLeft)
            drawRect(
                brush = Brush.horizontalGradient(
                    listOf(Color.Transparent, Color.Black.copy(alpha = depthAlpha * (foldFraction * 5f).coerceIn(0f, 1f))),
                    startX = backLeft, endX = foldX
                ),
                topLeft = Offset(backLeft, 0f),
                size = Size(foldX - backLeft, h)
            )
        }
    }
}

/**
 * Shadow overlay: drawn as the absolute TOP layer.
 *
 * Shadows cast FROM the fold line:
 *   - Right: onto the revealed next page
 *   - Left: onto the static left page (from back-flap leading edge)
 */
internal fun Modifier.drawBookSpreadShadow(
    foldFraction: Float,
    shadowAlpha: Float,
): Modifier = drawWithCache {
    val w = size.width
    val h = size.height
    val spineX = w / 2f
    val halfW = w / 2f
    val foldX = spineX + halfW * foldFraction
    val backLeft = (2f * foldX - w).coerceAtLeast(0f)

    val curShadowAlpha = shadowAlpha * (foldFraction * 5f).coerceIn(0f, 1f)
    if (foldFraction >= 0.999f || curShadowAlpha <= 0f) {
        return@drawWithCache onDrawWithContent { /* nothing */ }
    }

    val shadowW = 20.dp.toPx()

    onDrawWithContent {
        // Shadow at fold line, cast RIGHT onto the revealed page below
        val sRight = (foldX + shadowW).coerceAtMost(w)
        if (sRight - foldX > 1f) {
            drawRect(
                brush = Brush.horizontalGradient(
                    listOf(Color.Black.copy(alpha = curShadowAlpha * 0.5f), Color.Transparent),
                    startX = foldX, endX = sRight
                ),
                topLeft = Offset(foldX, 0f),
                size = Size(sRight - foldX, h)
            )
        }

        // Shadow at back-flap leading edge, cast LEFT onto the static left page
        if (backLeft > 1f) {
            val sLeftStart = (backLeft - shadowW * 0.7f).coerceAtLeast(0f)
            drawRect(
                brush = Brush.horizontalGradient(
                    listOf(Color.Transparent, Color.Black.copy(alpha = curShadowAlpha * 0.35f)),
                    startX = sLeftStart, endX = backLeft
                ),
                topLeft = Offset(sLeftStart, 0f),
                size = Size(backLeft - sLeftStart, h)
            )
        }

        // Thin dark line at the fold itself
        drawRect(
            color = Color.Black.copy(alpha = curShadowAlpha * 0.3f),
            topLeft = Offset(foldX - 0.5f, 0f),
            size = Size(1f, h)
        )
    }
}
